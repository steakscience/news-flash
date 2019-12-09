use crate::gtk_handle;
use crate::util::{BuilderHelper, GtkHandle};
use gtk::{Box, Button, ButtonExt, HeaderBar, MenuButton, ToggleButton, WidgetExt};
use libhandy::{Leaflet, LeafletExt};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct ResponsiveLayout {
    pub state: GtkHandle<ResponsiveState>,
    pub left_button: Button,
    pub search_button: ToggleButton,
    pub right_button: Button,
    pub major_leaflet: Leaflet,
    pub minor_leaflet: Leaflet,
    pub header_leaflet: Leaflet,
    pub sidebar_box: Box,
    pub article_list_box: Box,
    pub article_view_box: Box,
    pub left_header: HeaderBar,
    pub right_header: HeaderBar,
    pub mode_switch_box: Box,
    pub mode_switch_button: MenuButton,
    pub mark_all_read_button: Button,
}

impl ResponsiveLayout {
    pub fn new(builder: &BuilderHelper) -> Self {
        let state = gtk_handle!(ResponsiveState::new());

        let minor_leaflet = builder.get::<Leaflet>("minor_leaflet");
        let major_leaflet = builder.get::<Leaflet>("major_leaflet");
        let left_button = builder.get::<Button>("left_back_button");
        let search_button = builder.get::<ToggleButton>("search_button");
        let right_button = builder.get::<Button>("right_back_button");
        let sidebar_box = builder.get::<Box>("feedlist_box");
        let article_list_box = builder.get::<Box>("articlelist_box");
        let article_view_box = builder.get::<Box>("articleview_box");
        let header_leaflet = builder.get::<Leaflet>("header_leaflet");
        let left_header = builder.get::<HeaderBar>("left_headerbar");
        let right_header = builder.get::<HeaderBar>("right_headerbar");
        let mode_switch_box = builder.get::<Box>("mode_switch_box");
        let mode_switch_button = builder.get::<MenuButton>("mode_switch_button");
        let mark_all_read_button = builder.get::<Button>("mark_all_button");
        let layout = ResponsiveLayout {
            state,
            left_button,
            search_button,
            right_button,
            major_leaflet,
            minor_leaflet,
            header_leaflet,
            sidebar_box,
            article_list_box,
            article_view_box,
            left_header,
            right_header,
            mode_switch_box,
            mode_switch_button,
            mark_all_read_button,
        };
        layout.setup_signals();
        layout
    }

    fn setup_signals(&self) {
        let major_leaflet_layout = self.clone();
        self.major_leaflet.connect_property_folded_notify(move |_leaflet| {
            major_leaflet_layout.state.borrow_mut().major_leaflet_folded = true;
            Self::process_state_change(&major_leaflet_layout);
        });

        let minor_leaflet_layout = self.clone();
        self.minor_leaflet.connect_property_folded_notify(move |_leaflet| {
            minor_leaflet_layout.state.borrow_mut().minor_leaflet_folded = true;
            Self::process_state_change(&minor_leaflet_layout);
        });

        let left_back_button_layout = self.clone();
        self.left_button.connect_clicked(move |_button| {
            left_back_button_layout.state.borrow_mut().left_button_clicked = true;
            Self::process_state_change(&left_back_button_layout);
        });

        let right_back_button_layout = self.clone();
        self.right_button.connect_clicked(move |_button| {
            right_back_button_layout.state.borrow_mut().right_button_clicked = true;
            Self::process_state_change(&right_back_button_layout);
        });
    }

    pub fn process_state_change(&self) {
        if self.state.borrow().major_leaflet_folded {
            // article view (dis)appeared
            if !self.major_leaflet.get_property_folded() {
                self.right_button.set_visible(false);
                self.major_leaflet.set_visible_child(&self.minor_leaflet);
                self.header_leaflet.set_visible_child(&self.left_header);
                self.mode_switch_box.set_visible(true);
                self.mode_switch_button.set_visible(false);
            } else {
                self.mode_switch_box.set_visible(false);
                self.mode_switch_button.set_visible(true);
            }

            self.state.borrow_mut().major_leaflet_folded = false;
            return;
        }

        if self.state.borrow().minor_leaflet_folded {
            // article list (dis)appeared
            if !self.minor_leaflet.get_property_folded() {
                self.left_button.set_visible(false);
                self.search_button.set_visible(true);
                self.mode_switch_button.set_visible(true);
                self.minor_leaflet.set_visible_child(&self.sidebar_box);
                self.mark_all_read_button.set_visible(true);
            } else {
                self.search_button.set_visible(false);
                self.mode_switch_button.set_visible(false);
                self.mark_all_read_button.set_visible(false);
            }

            self.state.borrow_mut().minor_leaflet_folded = false;
            return;
        }

        if self.state.borrow().left_button_clicked {
            // left back
            self.minor_leaflet.set_visible_child(&self.sidebar_box);
            self.left_button.set_visible(false);
            self.search_button.set_visible(false);
            self.mode_switch_button.set_visible(false);
            self.mark_all_read_button.set_visible(false);

            self.state.borrow_mut().left_button_clicked = false;
            return;
        }

        if self.state.borrow().right_button_clicked {
            // right back
            self.major_leaflet.set_visible_child(&self.minor_leaflet);
            self.header_leaflet.set_visible_child(&self.left_header);
            self.right_button.set_visible(false);

            self.state.borrow_mut().right_button_clicked = false;
            return;
        }

        if self.state.borrow().major_leaflet_selected {
            // article selected
            if self.major_leaflet.get_property_folded() {
                self.major_leaflet.set_visible_child(&self.article_view_box);
                self.header_leaflet.set_visible_child(&self.right_header);
                self.right_button.set_visible(true);
            }

            self.state.borrow_mut().major_leaflet_selected = false;
            return;
        }

        if self.state.borrow().minor_leaflet_selected {
            // sidebar selected
            if self.minor_leaflet.get_property_folded() {
                self.minor_leaflet.set_visible_child(&self.article_list_box);
                self.left_button.set_visible(true);
                self.search_button.set_visible(true);
                self.mode_switch_button.set_visible(true);
                self.mark_all_read_button.set_visible(true);
            }

            self.state.borrow_mut().minor_leaflet_selected = false;
            return;
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResponsiveState {
    pub left_button_clicked: bool,
    pub right_button_clicked: bool,
    pub major_leaflet_selected: bool,
    pub major_leaflet_folded: bool,
    pub minor_leaflet_selected: bool,
    pub minor_leaflet_folded: bool,
}

impl ResponsiveState {
    pub fn new() -> Self {
        ResponsiveState {
            left_button_clicked: false,
            right_button_clicked: false,
            major_leaflet_selected: false,
            major_leaflet_folded: false,
            minor_leaflet_selected: false,
            minor_leaflet_folded: false,
        }
    }
}
