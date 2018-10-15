use Resources;
use std::str;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use failure::ResultExt;
use news_flash::models::{
    CategoryID,
    FeedID,
};
use sidebar::{
    CategoryRow,
    FeedRow,
    feed_list::models::{
        FeedListTree,
        FeedListCategoryModel,
        FeedListFeedModel,
        FeedListChangeSet,
    },
};
use gtk::{
    self,
    ListBoxExt,
    ListBoxRowExt,
    StyleContextExt,
    ContainerExt,
    WidgetExt,
    WidgetExtManual,
    DestDefaults,
    TargetFlags,
    TargetEntry,
    Inhibit,
};
use gdk::{
    EventType,
    DragAction,
};
use sidebar::feed_list::error::{
    FeedListError,
    FeedListErrorKind,
};

type Handle<T> = Rc<RefCell<T>>;
type HandleMap<T, K> = Handle<HashMap<T, K>>;


#[derive(Clone, Debug)]
pub struct FeedList {
    pub(crate) widget: gtk::ListBox,
    categories: HandleMap<CategoryID, Handle<CategoryRow>>,
    feeds: HandleMap<FeedID, Handle<FeedRow>>,
    tree: Handle<FeedListTree>,
}

impl FeedList {
    pub fn new() -> Result<Self, FeedListError> {
        let ui_data = Resources::get("ui/feedlist.ui").ok_or(FeedListErrorKind::UIFile)?;
        let ui_string = str::from_utf8(ui_data.as_ref()).context(FeedListErrorKind::EmbedFile)?;
        let builder = gtk::Builder::new_from_string(ui_string);
        let feed_list : gtk::ListBox = builder.get_object("feed_list").ok_or(FeedListErrorKind::UIFile)?;
        let entry = TargetEntry::new("FeedRow", TargetFlags::SAME_APP, 0);
        feed_list.drag_dest_set(DestDefaults::DROP | DestDefaults::MOTION, &vec![entry], DragAction::MOVE);
        feed_list.connect_drag_data_received(|_widget, _drag_context, _x, _y, _selection_data, _info, _time| {

        });
        feed_list.connect_drag_motion(|widget, _drag_context, _x, y, _time| {
            if let Some(row) = widget.get_row_at_y(y) {
                let alloc = row.get_allocation();
                let index = row.get_index();

                let (row_before, row_after) = match y < alloc.y + (alloc.height / 2) {
                    true => {
                        let row_before = widget.get_row_at_index(index - 1).unwrap();
                        (row_before, row)
                    },
                    false => {
                        let row_after = widget.get_row_at_index(index + 1).unwrap();
                        (row, row_after)
                    },
                };
                let style_context_before = row_before.get_style_context().unwrap();
                let style_context_after = row_after.get_style_context().unwrap();
                style_context_before.add_class("feedlist-drag-before");
                style_context_after.add_class("feedlist-drag-after");
            }
            
            Inhibit(false)
        });
        feed_list.connect_drag_leave(|_widget, _drag_context, _time| {

        });

        Ok(FeedList {
            widget: feed_list,
            categories: Rc::new(RefCell::new(HashMap::new())),
            feeds: Rc::new(RefCell::new(HashMap::new())),
            tree: Rc::new(RefCell::new(FeedListTree::new())),
        })
    }

    pub fn update(&mut self, new_tree: FeedListTree) {
        let old_tree = self.tree.clone();
        self.tree = Rc::new(RefCell::new(new_tree));
        let tree_diff = old_tree.borrow().generate_diff(&self.tree.borrow());
        for diff in tree_diff {
            match diff {
                FeedListChangeSet::RemoveFeed(feed_id) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&feed_id) {
                        self.widget.remove(&feed_handle.borrow().row());
                    }
                    self.feeds.borrow_mut().remove(&feed_id);
                },
                FeedListChangeSet::RemoveCategory(category_id) => {
                    if let Some(category_handle) = self.categories.borrow().get(&category_id) {
                        self.widget.remove(&category_handle.borrow().row());
                    }
                    self.categories.borrow_mut().remove(&category_id);
                },
                FeedListChangeSet::AddFeed(model, pos, visible) => {
                    self.add_feed(&model, pos, visible);
                },
                FeedListChangeSet::AddCategory(model, pos, visible) => {
                    self.add_category(&model, pos, visible);
                },
                FeedListChangeSet::FeedUpdateItemCount(id, count) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&id) {
                        feed_handle.borrow().update_item_count(count);
                    }
                },
                FeedListChangeSet::CategoryUpdateItemCount(id, count) => {
                    if let Some(category_handle) = self.categories.borrow().get(&id) {
                        category_handle.borrow().update_item_count(count);
                    }
                },
                FeedListChangeSet::FeedUpdateLabel(id, label) => {
                    if let Some(feed_handle) = self.feeds.borrow().get(&id) {
                        feed_handle.borrow().update_title(&label);
                    }
                },
                FeedListChangeSet::CategoryUpdateLabel(id, label) => {
                    if let Some(category_handle) = self.categories.borrow().get(&id) {
                        category_handle.borrow().update_title(&label);
                    }
                },
            }
        }
    }

    fn add_category(&mut self, category: &FeedListCategoryModel, pos: i32, visible: bool) {
        let category_widget = CategoryRow::new(category, visible);
        let feeds = self.feeds.clone();
        let categories = self.categories.clone();
        let category_id = category.id.clone();
        let tree = self.tree.clone();
        self.widget.insert(&category_widget.borrow().row(), pos);
        self.categories.borrow_mut().insert(category.id.clone(), category_widget.clone());

        category_widget.borrow().expander_event().connect_button_press_event(move |_widget, event| {
            if event.get_event_type() == EventType::ButtonPress {
                if let Some((feed_ids, category_ids, expaneded)) = tree.borrow_mut().collapse_expand_ids(&category_id) {
                    for feed_id in feed_ids {
                        if let Some(feed_handle) = feeds.borrow().get(&feed_id) {
                            if expaneded {
                                feed_handle.borrow_mut().expand();
                            }
                            else {
                                feed_handle.borrow_mut().collapse();
                            }
                        }
                    }
                    for category_id in category_ids {
                        if let Some(category_handle) = categories.borrow().get(&category_id) {
                            if expaneded {
                                category_handle.borrow_mut().expand();
                            }
                            else {
                                category_handle.borrow_mut().collapse();
                            }
                        }
                    }
                }
            }
            gtk::Inhibit(true)
        });
    }

    fn add_feed(&mut self, feed: &FeedListFeedModel, pos: i32, visible: bool) {
        let feed_widget = FeedRow::new(feed, visible);
        self.widget.insert(&feed_widget.borrow().row(), pos);
        self.feeds.borrow_mut().insert(feed.id.clone(), feed_widget);
    }
}