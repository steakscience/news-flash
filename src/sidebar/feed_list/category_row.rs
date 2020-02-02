use crate::app::Action;
use crate::main_window_state::MainWindowState;
use crate::sidebar::feed_list::models::FeedListCategoryModel;
use crate::undo_bar::UndoActionModel;
use crate::util::{BuilderHelper, GtkUtil, Util};
use gdk::{EventMask, EventType};
use gio::{ActionMapExt, Menu, MenuItem, SimpleAction};
use glib::{clone, object::IsA, Sender};
use gtk::{
    self, prelude::WidgetExtManual, BinExt, Box, ContainerExt, EventBox, Image, Inhibit, Label, LabelExt, ListBoxRow,
    ListBoxRowExt, Popover, PopoverExt, PositionType, Revealer, RevealerExt, StateFlags, StyleContextExt, Widget,
    WidgetExt,
};
use news_flash::models::CategoryID;
use parking_lot::RwLock;
use std::str;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct CategoryRow {
    pub id: CategoryID,
    widget: ListBoxRow,
    revealer: Revealer,
    arrow_event: EventBox,
    item_count: Label,
    item_count_event: EventBox,
    title: Label,
    expanded: bool,
}

impl CategoryRow {
    pub fn new(
        model: &FeedListCategoryModel,
        state: &Arc<RwLock<MainWindowState>>,
        visible: bool,
        sender: Sender<Action>,
    ) -> Arc<RwLock<CategoryRow>> {
        let builder = BuilderHelper::new("category");
        let revealer = builder.get::<Revealer>("category_row");
        let level_margin = builder.get::<Box>("level_margin");
        level_margin.set_margin_start(model.level * 24);

        let title_label = builder.get::<Label>("category_title");
        let item_count_label = builder.get::<Label>("item_count");
        let item_count_event = builder.get::<EventBox>("item_count_event");
        let arrow_image = builder.get::<Image>("arrow_image");

        let arrow_event = builder.get::<EventBox>("arrow_event");
        let category = CategoryRow {
            id: model.id.clone(),
            widget: Self::create_row(&revealer, &model.id, &title_label, state, &sender),
            revealer,
            arrow_event: arrow_event.clone(),
            item_count: item_count_label,
            item_count_event,
            title: title_label.clone(),
            expanded: model.expanded,
        };
        category.update_title(&model.label);
        category.update_item_count(model.item_count);
        Self::rotate_arrow(&arrow_image, model.expanded);
        if !visible {
            category.collapse();
        }
        let handle = Arc::new(RwLock::new(category));
        let handle1 = handle.clone();

        arrow_event.set_events(EventMask::BUTTON_PRESS_MASK);
        arrow_event.set_events(EventMask::ENTER_NOTIFY_MASK);
        arrow_event.set_events(EventMask::LEAVE_NOTIFY_MASK);

        arrow_event.connect_enter_notify_event(clone!(@weak arrow_image => @default-panic, move |_widget, _| {
            arrow_image.set_opacity(1.0);
            Inhibit(false)
        }));

        arrow_event.connect_leave_notify_event(clone!(@weak arrow_image => @default-panic, move |_widget, _| {
            arrow_image.set_opacity(0.8);
            Inhibit(false)
        }));

        arrow_event.connect_button_press_event(clone!(@weak arrow_image => @default-panic, move |_widget, event| {
            if event.get_button() != 1 {
                return Inhibit(false);
            }
            match event.get_event_type() {
                EventType::ButtonPress => (),
                _ => return gtk::Inhibit(false),
            }
            let expanded = handle1.read().expanded;
            Self::rotate_arrow(&arrow_image, !expanded);
            handle1.write().expanded = !expanded;
            Inhibit(false)
        }));

        handle
    }

    fn rotate_arrow<W: IsA<Widget>>(arrow_image: &W, expanded: bool) {
        let context = arrow_image.get_style_context();

        if expanded {
            context.remove_class("forward-arrow-collapsed");
            context.add_class("forward-arrow-expanded");
        } else {
            context.remove_class("forward-arrow-expanded");
            context.add_class("forward-arrow-collapsed");
        }
    }

    fn create_row(
        widget: &Revealer,
        id: &CategoryID,
        label: &Label,
        state: &Arc<RwLock<MainWindowState>>,
        sender: &Sender<Action>,
    ) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.get_style_context().remove_class("activatable");

        let eventbox = EventBox::new();
        eventbox.set_events(EventMask::BUTTON_PRESS_MASK);

        row.add(&eventbox);
        eventbox.add(widget);

        eventbox.connect_button_press_event(clone!(
            @strong id as category_id,
            @strong sender,
            @weak label,
            @weak state,
            @weak row => @default-panic, move |_eventbox, event| {
            if event.get_button() != 3 {
                return Inhibit(false);
            }

            match event.get_event_type() {
                EventType::ButtonRelease | EventType::DoubleButtonPress | EventType::TripleButtonPress => {
                    return Inhibit(false)
                }
                _ => {}
            }

            if state.read().get_offline() {
                return Inhibit(false);
            }

            let model = Menu::new();

            let rename_category_dialog_action = SimpleAction::new("rename-category-dialog", None);
            rename_category_dialog_action.connect_activate(clone!(@strong sender, @strong category_id => move |_action, _parameter| {
                Util::send(&sender, Action::RenameCategoryDialog(category_id.clone()));
            }));

            if let Ok(main_window) = GtkUtil::get_main_window(&row) {
                main_window.add_action(&rename_category_dialog_action);
            }

            let rename_category_item = MenuItem::new(Some("Rename"), None);
            rename_category_item.set_action_and_target_value(Some("rename-category-dialog"), None);
            model.append_item(&rename_category_item);

            let delete_category_item = MenuItem::new(Some("Delete"), None);
            let delete_category_action = SimpleAction::new("enqueue-delete-category", None);
            delete_category_action.connect_activate(clone!(
                @weak label,
                @strong category_id,
                @strong sender => move |_action, _parameter|
            {
                let label = match label.get_text() {
                    Some(label) => label.as_str().to_owned(),
                    None => "".to_owned(),
                };
                let remove_action = UndoActionModel::DeleteCategory((category_id.clone(), label));
                Util::send(&sender, Action::UndoableAction(remove_action));
            }));
            if let Ok(main_window) = GtkUtil::get_main_window(&row) {
                main_window.add_action(&delete_category_action);
            }
            delete_category_item.set_action_and_target_value(Some("enqueue-delete-category"), None);
            model.append_item(&delete_category_item);

            let popover = Popover::new(Some(&row));
            popover.set_position(PositionType::Bottom);
            popover.bind_model(Some(&model), Some("win"));
            popover.show();
            popover.connect_closed(clone!(@weak row => move |_popover| {
                row.unset_state_flags(StateFlags::PRELIGHT);
            }));
            row.set_state_flags(StateFlags::PRELIGHT, false);

            Inhibit(true)
        }));

        row
    }

    pub fn widget(&self) -> ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i64) {
        if count > 0 {
            self.item_count.set_label(&count.to_string());
            self.item_count_event.set_visible(true);
        } else {
            self.item_count_event.set_visible(false);
        }
    }

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }

    pub fn expander_event(&self) -> EventBox {
        self.arrow_event.clone()
    }

    pub fn expand_collapse_arrow(&mut self) {
        self.expanded = !self.expanded;
        let arrow_image = self
            .arrow_event
            .get_child()
            .expect("arrow_image is not child of arrow_event");
        Self::rotate_arrow(&arrow_image, self.expanded);
    }

    pub fn collapse(&self) {
        self.revealer.set_reveal_child(false);
        self.revealer.get_style_context().add_class("hidden");
        self.widget.set_selectable(false);
    }

    pub fn expand(&self) {
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().remove_class("hidden");
        self.widget.set_selectable(true);
    }
}
