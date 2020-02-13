use crate::app::{Action, App};
use crate::settings::Settings;
use crate::util::{BuilderHelper, GtkUtil, Util, CHANNEL_ERROR, RUNTIME_ERROR};
use feedly_api::models::SearchResultItem;
use futures::channel::oneshot;
use futures::executor::ThreadPool;
use futures::FutureExt;
use gdk::NotifyType;
use glib::{clone, Sender};
use gtk::{
    Button, ButtonExt, ContainerExt, EventBox, Image, ImageExt, Inhibit, Label, LabelExt, ListBoxRow, ListBoxRowExt,
    Stack, StackExt, StyleContextExt, WidgetExt,
};
use news_flash::models::Url;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct SearchItemRow {
    pub widget: ListBoxRow,
}

impl SearchItemRow {
    pub fn new(
        item: &SearchResultItem,
        settings: &Arc<RwLock<Settings>>,
        threadpool: &ThreadPool,
        sender: &Sender<Action>,
        is_last: bool,
    ) -> Self {
        let builder = BuilderHelper::new("discover_dialog");
        let search_item_row = builder.get::<EventBox>("search_item_row");
        let search_item_title = builder.get::<Label>("search_item_title");
        let search_item_description = builder.get::<Label>("search_item_description");
        let search_item_image = builder.get::<Image>("search_item_image");
        let subscribe_stack = builder.get::<Stack>("subscribe_stack");
        let subscribe_button = builder.get::<Button>("subscribe_button");

        let search_item_feed_url = Arc::new(RwLock::new(Self::feedly_id_to_rss_url(&item.feed_id)));
        subscribe_button.set_sensitive(false);
        subscribe_button.connect_clicked(clone!(@strong sender, @strong search_item_feed_url => move |_button| {
            if let Some(feed_url) = search_item_feed_url.read().as_ref() {
                Util::send(&sender, Action::AddFeed((feed_url.clone(), None, None)));
            }
        }));

        search_item_row.connect_leave_notify_event(
            clone!(@weak subscribe_stack => @default-panic, move |_widget, event| {
                if event.get_detail() == NotifyType::Inferior {
                    return Inhibit(false);
                }
                subscribe_stack.set_visible_child_name("empty");
                gtk::Inhibit(false)
            }),
        );
        search_item_row.connect_enter_notify_event(
            clone!(@weak subscribe_stack => @default-panic, move |_widget, event| {
                if event.get_detail() == NotifyType::Inferior {
                    return Inhibit(false);
                }
                subscribe_stack.set_visible_child_name("button");
                gtk::Inhibit(false)
            }),
        );

        let scale = GtkUtil::get_scale(&search_item_image);

        search_item_title.set_label(
            &item
                .title
                .clone()
                .expect("Empty titles should not be created in the first place!"),
        );

        let description = if let Some(description) = &item.description {
            let description = str::replace(&description, "\n", " ");
            let description = str::replace(&description, "\r", " ");
            let description = str::replace(&description, "_", " ");
            description
        } else {
            "No description".to_owned()
        };

        search_item_description.set_label(&description);

        let row = ListBoxRow::new();
        row.set_activatable(true);
        row.set_can_focus(false);
        row.add(&search_item_row);
        row.show_all();
        let context = row.get_style_context();
        context.remove_class("activatable");
        if !is_last {
            context.add_class("search-item-separator");
        }

        let icon_url = if let Some(visual_url) = &item.visual_url {
            Some(visual_url.clone())
        } else if let Some(logo) = &item.logo {
            Some(logo.clone())
        } else if let Some(icon_url) = &item.icon_url {
            Some(icon_url.clone())
        } else {
            None
        };

        if let Some(icon_url) = icon_url {
            let (sender, receiver) = oneshot::channel::<Option<Vec<u8>>>();

            let settings = settings.clone();
            let thread_future = async move {
                let mut runtime = Runtime::new().expect(RUNTIME_ERROR);
                let client = App::build_client(&settings);

                let res = match runtime.block_on(client.get(&icon_url).send()) {
                    Ok(response) => match runtime.block_on(response.bytes()) {
                        Ok(bytes) => Some(Vec::from(bytes.as_ref())),
                        Err(_) => None,
                    },
                    Err(_) => None,
                };
                sender.send(res).expect(CHANNEL_ERROR);
            };

            let glib_future = receiver.map(clone!(@strong search_item_image => @default-panic, move |res| {
                if let Some(byte_vec) = res.expect(CHANNEL_ERROR) {
                    if let Ok(surface) = GtkUtil::create_surface_from_bytes(&byte_vec, 64, 64, scale) {
                        search_item_image.set_from_surface(Some(&surface));
                    }
                }
            }));

            threadpool.spawn_ok(thread_future);
            Util::glib_spawn_future(glib_future);
        }

        SearchItemRow { widget: row }
    }

    fn feedly_id_to_rss_url(feedly_id: &str) -> Option<Url> {
        let url_string: String = feedly_id.chars().skip(5).collect();
        if let Ok(url) = Url::parse(&url_string) {
            Some(url)
        } else {
            None
        }
    }
}
