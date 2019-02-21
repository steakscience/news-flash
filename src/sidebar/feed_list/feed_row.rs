use gtk::{
    self,
    LabelExt,
    ContainerExt,
    WidgetExt,
    WidgetExtManual,
    StyleContextExt,
    ListBoxRowExt,
    RevealerExt,
    TargetEntry,
    TargetFlags,
    DragContextExtManual,
    ImageExt,
};
use gdk::{
    DragAction,
    ModifierType,
};
use cairo::{
    self,
    ImageSurface,
    Format,
};
use news_flash::models::{
    FeedID,
    FavIcon,
    PixelIcon,
};
use crate::sidebar::feed_list::models::{
    FeedListFeedModel,
};
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use crate::Resources;
use crate::gtk_util::GtkUtil;

#[derive(Clone, Debug)]
pub struct FeedRow {
    pub id: FeedID,
    widget: gtk::ListBoxRow,
    item_count: gtk::Label,
    item_count_event: gtk::EventBox,
    title: gtk::Label,
    revealer: gtk::Revealer,
    favicon: gtk::Image,
}

impl FeedRow {
    pub fn new(model: &FeedListFeedModel, visible: bool) -> Rc<RefCell<FeedRow>> {
        let ui_data = Resources::get("ui/feed.ui").unwrap();
        let ui_string = str::from_utf8(ui_data.as_ref()).unwrap();
        let builder = gtk::Builder::new_from_string(ui_string);
        let feed : gtk::Revealer = builder.get_object("feed_row").unwrap();
        feed.set_margin_start(model.level*24);
        
        let title_label : gtk::Label = builder.get_object("feed_title").unwrap();
        let item_count_label : gtk::Label = builder.get_object("item_count").unwrap();
        let item_count_event : gtk::EventBox = builder.get_object("item_count_event").unwrap();
        let favicon : gtk::Image = builder.get_object("favicon").unwrap();

        let feed = FeedRow {
            id: model.id.clone(),
            widget: Self::create_row(&feed, &model.id),
            item_count: item_count_label,
            title: title_label,
            revealer: feed,
            item_count_event: item_count_event,
            favicon: favicon,
        };
        feed.update_item_count(model.item_count);
        feed.update_title(&model.label);
        feed.update_favicon(&model.icon);
        if !visible {
            feed.collapse();
        }
        Rc::new(RefCell::new(feed))
    }

    fn create_row(widget: &gtk::Revealer, id: &FeedID) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.set_activatable(false);
        row.set_can_focus(false);
        row.add(widget);
        let context = row.get_style_context().unwrap();
        context.remove_class("activatable");
        let row_2nd_handle = row.clone();
        let id = id.clone();

        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        widget.drag_source_set(ModifierType::BUTTON1_MASK, &vec![entry], DragAction::MOVE);
        widget.drag_source_add_text_targets();
        widget.connect_drag_data_get(move |_widget, _ctx, selection_data, _info, _time| {
            if let Ok(json) = serde_json::to_string(&id.clone()) {
                let mut data =  String::from("FeedID ");
                data.push_str(&json);
                selection_data.set_text(&data);
            }
        });
        widget.connect_drag_begin(move |_widget, drag_context| {
            let alloc = row.get_allocation();
            let surface = ImageSurface::create(Format::ARgb32, alloc.width, alloc.height).unwrap();
            let cairo_context = cairo::Context::new(&surface);
            let style_context = row.get_style_context().unwrap();
            style_context.add_class("drag-icon");
            row.draw(&cairo_context);
            style_context.remove_class("drag-icon");
            drag_context.drag_set_icon_surface(&surface);
        });
        
        row_2nd_handle
    }
    
    pub fn row(&self) -> gtk::ListBoxRow {
        self.widget.clone()
    }

    pub fn update_item_count(&self, count: i32) {
        if count > 0 {
            self.item_count.set_label(&count.to_string());
            self.item_count_event.set_visible(true);
        }
        else {
            self.item_count_event.set_visible(false);
        }
    }

    pub fn update_favicon(&self, icon: &Option<FavIcon>) {
        if let Some(icon) = icon {
            // FIXME: deal with different icon formats and sizes
            if let Some(data) = &icon.data {
                let icon = PixelIcon {
                    data: data.clone(),
                    width: 16,
                    height: 16,
                    has_alpha: true,
                    bits_per_sample: 8,
                    row_stride: 512,
                };
                let surface = GtkUtil::create_surface_from_bitmap(&icon, 1).unwrap();
                self.favicon.set_from_surface(&surface);
            }
        }
    }

    pub fn update_title(&self, title: &str) {
        self.title.set_label(title);
    }

    pub fn collapse(&self) {
        self.revealer.set_reveal_child(false);
        self.revealer.get_style_context().unwrap().add_class("hidden");
        self.widget.set_selectable(false);
    }

    pub fn expand(&self) {
        self.revealer.set_reveal_child(true);
        self.revealer.get_style_context().unwrap().remove_class("hidden");
        self.widget.set_selectable(true);
    }
}
