// The tray button as a Waybar CFFI module: a chevron that points down while
// the tray is closed and turns sideways, on a spring, when it opens.
// Built by install.sh into ~/.cache/octopus/tray-arrow.so.

#define _GNU_SOURCE
#include <dlfcn.h>
#include <gtk/gtk.h>
#include <math.h>
#include <string.h>

typedef struct {
  void *obj;
  const char *waybar_version;
  GtkContainer *(*get_root_widget)(void *obj);
  void (*queue_update)(void *obj);
} wbcffi_init_info;

typedef struct {
  const char *key;
  const char *value;
} wbcffi_config_entry;

const size_t wbcffi_version = 2;

typedef struct {
  GtkWidget *events;
  GtkWidget *box;
  GtkWidget *area;
  double angle, speed, target;
  guint tick;
  gint64 last;
  gboolean right;
  // Waybar destroys the widgets on every reload while a status request may
  // still be on its way, so the state outlives them until the last answer.
  int refs;
  gboolean dead;
} Arrow;

static void unref(Arrow *a) {
  if (--a->refs == 0) g_free(a);
}

static void gone(GtkWidget *widget, gpointer data) {
  Arrow *a = data;
  a->dead = TRUE;
  a->tick = 0;
  a->events = a->box = a->area = NULL;
  unref(a);
}

// Waybar may unload the module on reload while an answer is still due;
// staying loaded keeps that callback's code around.
static void stay_loaded(void) {
  Dl_info info;
  if (dladdr((void *)stay_loaded, &info) && info.dli_fname)
    dlopen(info.dli_fname, RTLD_NOW | RTLD_NOLOAD | RTLD_NODELETE);
}

static const double STIFFNESS = 420.0;
static const double DAMPING = 17.0;

static gboolean step(GtkWidget *widget, GdkFrameClock *clock, gpointer data) {
  Arrow *a = data;
  if (a->dead) return G_SOURCE_REMOVE;
  gint64 now = gdk_frame_clock_get_frame_time(clock);
  double dt = a->last ? (now - a->last) / 1e6 : 0.0;
  a->last = now;
  if (dt > 0.05) dt = 0.05;
  for (int i = 0; i < 4; i++) {
    double h = dt / 4;
    a->speed += (-STIFFNESS * (a->angle - a->target) - DAMPING * a->speed) * h;
    a->angle += a->speed * h;
  }
  gtk_widget_queue_draw(a->area);
  if (fabs(a->angle - a->target) < 0.001 && fabs(a->speed) < 0.01) {
    a->angle = a->target;
    a->speed = 0;
    a->tick = 0;
    a->last = 0;
    return G_SOURCE_REMOVE;
  }
  return G_SOURCE_CONTINUE;
}

static void turn(Arrow *a, gboolean open) {
  if (a->dead) return;
  double target = open ? (a->right ? M_PI / 2 : -M_PI / 2) : 0.0;
  if (target == a->target) return;
  a->target = target;
  if (!a->tick) a->tick = gtk_widget_add_tick_callback(a->area, step, a, NULL);
}

static gboolean draw(GtkWidget *widget, cairo_t *cr, gpointer data) {
  Arrow *a = data;
  if (a->dead) return FALSE;
  int w = gtk_widget_get_allocated_width(widget);
  int h = gtk_widget_get_allocated_height(widget);
  GtkStyleContext *style = gtk_widget_get_style_context(a->box);
  GdkRGBA color;
  gtk_style_context_get_color(style, gtk_style_context_get_state(style), &color);

  PangoLayout *layout = gtk_widget_create_pango_layout(widget, "keyboard_arrow_down");
  PangoFontDescription *font = pango_font_description_from_string("Material Symbols Rounded 13");
  pango_layout_set_font_description(layout, font);
  int lw, lh;
  pango_layout_get_pixel_size(layout, &lw, &lh);

  cairo_translate(cr, w / 2.0, h / 2.0);
  cairo_rotate(cr, a->angle);
  cairo_move_to(cr, -lw / 2.0, -lh / 2.0);
  gdk_cairo_set_source_rgba(cr, &color);
  pango_cairo_show_layout(cr, layout);

  pango_font_description_free(font);
  g_object_unref(layout);
  return FALSE;
}

static void apply(Arrow *a, const char *json) {
  if (a->dead) return;
  if (!json || strstr(json, "\"text\":\"\"") || strstr(json, "\"text\": \"\"")) {
    gtk_widget_hide(a->events);
    return;
  }
  gtk_widget_show_all(a->events);
  turn(a, strstr(json, "keyboard_arrow_down") == NULL);
  const char *tip = strstr(json, "\"tooltip\":");
  if (tip) {
    tip = strchr(tip + 10, '"');
    const char *end = tip ? strchr(tip + 1, '"') : NULL;
    if (end) {
      char *text = g_strndup(tip + 1, end - tip - 1);
      gtk_widget_set_tooltip_text(a->events, text);
      g_free(text);
    }
  }
}

static void answered(GObject *source, GAsyncResult *result, gpointer data) {
  Arrow *a = data;
  char *out = NULL;
  if (g_subprocess_communicate_utf8_finish(G_SUBPROCESS(source), result, &out, NULL, NULL))
    apply(a, out);
  g_free(out);
  unref(a);
}

static void ask(Arrow *a) {
  if (a->dead) return;
  GSubprocess *process = g_subprocess_new(G_SUBPROCESS_FLAGS_STDOUT_PIPE | G_SUBPROCESS_FLAGS_STDERR_SILENCE,
                                          NULL, "octopus", "tray-status", NULL);
  if (!process) return;
  a->refs++;
  g_subprocess_communicate_utf8_async(process, NULL, NULL, answered, a);
  g_object_unref(process);
}

static gboolean pressed(GtkWidget *widget, GdkEventButton *event, gpointer data) {
  if (event->button != 1) return FALSE;
  Arrow *a = data;
  turn(a, a->target == 0.0);
  g_spawn_command_line_async("octopus toggle tray", NULL);
  return TRUE;
}

static gboolean crossed(GtkWidget *widget, GdkEventCrossing *event, gpointer data) {
  Arrow *a = data;
  if (a->dead) return FALSE;
  GtkStyleContext *style = gtk_widget_get_style_context(a->box);
  if (event->type == GDK_ENTER_NOTIFY)
    gtk_style_context_add_class(style, "hover");
  else if (event->detail != GDK_NOTIFY_INFERIOR)
    gtk_style_context_remove_class(style, "hover");
  gtk_widget_queue_draw(a->area);
  return FALSE;
}

void *wbcffi_init(const wbcffi_init_info *info, const wbcffi_config_entry *entries, size_t count) {
  stay_loaded();
  Arrow *a = g_new0(Arrow, 1);
  // One for Waybar's instance, one for the widgets.
  a->refs = 2;
  for (size_t i = 0; i < count; i++)
    if (!strcmp(entries[i].key, "position")) a->right = strstr(entries[i].value, "right") != NULL;

  a->events = gtk_event_box_new();
  a->box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
  gtk_widget_set_name(a->box, "tray-arrow");
  a->area = gtk_drawing_area_new();
  gtk_widget_set_size_request(a->area, 24, 24);
  gtk_widget_set_hexpand(a->area, TRUE);
  gtk_container_add(GTK_CONTAINER(a->box), a->area);
  gtk_container_add(GTK_CONTAINER(a->events), a->box);
  gtk_container_add(info->get_root_widget(info->obj), a->events);

  g_signal_connect(a->area, "draw", G_CALLBACK(draw), a);
  g_signal_connect(a->events, "destroy", G_CALLBACK(gone), a);
  g_signal_connect(a->events, "button-press-event", G_CALLBACK(pressed), a);
  g_signal_connect(a->events, "enter-notify-event", G_CALLBACK(crossed), a);
  g_signal_connect(a->events, "leave-notify-event", G_CALLBACK(crossed), a);

  gtk_widget_hide(a->events);
  ask(a);
  return a;
}

void wbcffi_deinit(void *instance) {
  Arrow *a = instance;
  if (!a->dead && a->tick) gtk_widget_remove_tick_callback(a->area, a->tick);
  a->dead = TRUE;
  unref(a);
}

void wbcffi_refresh(void *instance, int signal) { ask(instance); }
