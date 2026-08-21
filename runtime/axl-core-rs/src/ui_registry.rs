use std::collections::HashMap;

pub struct PropertyContract {
    pub id: i32,
    pub type_name: &'static str,
}

pub struct ComponentContract {
    pub name: &'static str,
    pub properties: Vec<PropertyContract>,
    pub events: Vec<i32>,
    pub children: bool,
}

pub fn component(id: i32) -> Option<&'static ComponentContract> {
    COMPONENTS.get(&id)
}

pub fn component_by_name(name: &str) -> Option<(&'static ComponentContract, i32)> {
    COMPONENTS.iter().find(|(_, c)| c.name == name).map(|(id, c)| (c, *id))
}

pub fn annotation_kind_valid(kind: i32) -> bool {
    matches!(kind, 1 | 2 | 3)
}

pub fn all_components() -> Vec<(i32, &'static ComponentContract)> {
    COMPONENTS.iter().map(|(id, c)| (*id, c)).collect()
}

lazy_static::lazy_static! {
    static ref COMPONENTS: HashMap<i32, ComponentContract> = {
        let mut m = HashMap::new();

        // ====================================================================
        // LAYOUT
        // ====================================================================

        // 1: App — root container
        m.insert(1, ComponentContract {
            name: "app",
            properties: vec![PropertyContract { id: 1, type_name: "string" }],
            events: vec![],
            children: true,
        });

        // 10: Container — generic wrapper
        m.insert(10, ComponentContract {
            name: "container",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // class
                PropertyContract { id: 2, type_name: "string" },  // style
            ],
            events: vec![],
            children: true,
        });

        // 11: Grid — CSS grid layout
        m.insert(11, ComponentContract {
            name: "grid",
            properties: vec![
                PropertyContract { id: 1, type_name: "int" },    // columns
                PropertyContract { id: 2, type_name: "string" },  // gap
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![],
            children: true,
        });

        // 12: Flex — flexbox layout
        m.insert(12, ComponentContract {
            name: "flex",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // direction: row|column
                PropertyContract { id: 2, type_name: "string" },  // justify
                PropertyContract { id: 3, type_name: "string" },  // align
                PropertyContract { id: 4, type_name: "string" },  // gap
            ],
            events: vec![],
            children: true,
        });

        // 13: Divider — horizontal rule
        m.insert(13, ComponentContract {
            name: "divider",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // style: solid|dashed|dotted
                PropertyContract { id: 2, type_name: "string" },  // color
            ],
            events: vec![],
            children: false,
        });

        // ====================================================================
        // NAVIGATION
        // ====================================================================

        // 20: Navbar — top navigation
        m.insert(20, ComponentContract {
            name: "navbar",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // brand
                PropertyContract { id: 2, type_name: "string" },  // logo
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![1],  // on_navigate
            children: true,
        });

        // 21: Sidebar — side navigation
        m.insert(21, ComponentContract {
            name: "sidebar",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // position: left|right
                PropertyContract { id: 2, type_name: "int" },     // width
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![1],
            children: true,
        });

        // 22: Tabs — tab navigation
        m.insert(22, ComponentContract {
            name: "tabs",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // active tab
                PropertyContract { id: 2, type_name: "string" },  // style: line|pill|enclosed
            ],
            events: vec![1],  // on_tab_change
            children: true,
        });

        // 23: Breadcrumb — breadcrumb navigation
        m.insert(23, ComponentContract {
            name: "breadcrumb",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // separator
            ],
            events: vec![1],
            children: true,
        });

        // 24: Pagination — page navigation
        m.insert(24, ComponentContract {
            name: "pagination",
            properties: vec![
                PropertyContract { id: 1, type_name: "int" },     // current page
                PropertyContract { id: 2, type_name: "int" },     // total pages
                PropertyContract { id: 3, type_name: "int" },     // visible pages
            ],
            events: vec![1],  // on_page_change
            children: false,
        });

        // ====================================================================
        // CONTENT
        // ====================================================================

        // 30: Hero — hero section
        m.insert(30, ComponentContract {
            name: "hero",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "string" },  // subtitle
                PropertyContract { id: 3, type_name: "string" },  // description
                PropertyContract { id: 4, type_name: "string" },  // badge
                PropertyContract { id: 5, type_name: "string" },  // image
                PropertyContract { id: 6, type_name: "string" },  // video
                PropertyContract { id: 7, type_name: "string" },  // class
            ],
            events: vec![1, 2],  // on_primary, on_secondary
            children: true,
        });

        // 31: Shelf — horizontal scroll container
        m.insert(31, ComponentContract {
            name: "shelf",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "string" },  // subtitle
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![],
            children: true,
        });

        // 32: Media Card — content card
        m.insert(32, ComponentContract {
            name: "media-card",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "string" },  // subtitle
                PropertyContract { id: 3, type_name: "int" },     // tone (1-10)
                PropertyContract { id: 4, type_name: "int" },     // size (1-2)
                PropertyContract { id: 5, type_name: "string" },  // image
                PropertyContract { id: 6, type_name: "string" },  // badge
                PropertyContract { id: 7, type_name: "string" },  // class
            ],
            events: vec![1],  // on_click
            children: false,
        });

        // 33: Text Block — rich text
        m.insert(33, ComponentContract {
            name: "text-block",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // content
                PropertyContract { id: 2, type_name: "string" },  // tag: h1-h6|p|span
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![],
            children: false,
        });

        // 34: Image — image display
        m.insert(34, ComponentContract {
            name: "image",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // src
                PropertyContract { id: 2, type_name: "string" },  // alt
                PropertyContract { id: 3, type_name: "int" },     // width
                PropertyContract { id: 4, type_name: "int" },     // height
                PropertyContract { id: 5, type_name: "string" },  // class
            ],
            events: vec![1],  // on_click
            children: false,
        });

        // 35: Video — video player
        m.insert(35, ComponentContract {
            name: "video",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // src
                PropertyContract { id: 2, type_name: "string" },  // poster
                PropertyContract { id: 3, type_name: "bool" },    // autoplay
                PropertyContract { id: 4, type_name: "bool" },    // controls
            ],
            events: vec![1],  // on_play
            children: false,
        });

        // ====================================================================
        // FORMS
        // ====================================================================

        // 40: Input — text input
        m.insert(40, ComponentContract {
            name: "input",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // placeholder
                PropertyContract { id: 2, type_name: "string" },  // value
                PropertyContract { id: 3, type_name: "string" },  // type: text|password|email|number
                PropertyContract { id: 4, type_name: "string" },  // label
                PropertyContract { id: 5, type_name: "string" },  // class
            ],
            events: vec![1],  // on_change
            children: false,
        });

        // 41: Textarea — multi-line input
        m.insert(41, ComponentContract {
            name: "textarea",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // placeholder
                PropertyContract { id: 2, type_name: "string" },  // value
                PropertyContract { id: 3, type_name: "int" },     // rows
                PropertyContract { id: 4, type_name: "string" },  // label
            ],
            events: vec![1],
            children: false,
        });

        // 42: Select — dropdown
        m.insert(42, ComponentContract {
            name: "select",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // options (comma-separated)
                PropertyContract { id: 2, type_name: "string" },  // value
                PropertyContract { id: 3, type_name: "string" },  // label
            ],
            events: vec![1],
            children: false,
        });

        // 43: Checkbox — checkbox input
        m.insert(43, ComponentContract {
            name: "checkbox",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // label
                PropertyContract { id: 2, type_name: "bool" },    // checked
            ],
            events: vec![1],
            children: false,
        });

        // 44: Radio — radio input
        m.insert(44, ComponentContract {
            name: "radio",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // label
                PropertyContract { id: 2, type_name: "string" },  // value
                PropertyContract { id: 3, type_name: "bool" },    // checked
            ],
            events: vec![1],
            children: false,
        });

        // 45: Button — action button
        m.insert(45, ComponentContract {
            name: "button",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // label
                PropertyContract { id: 2, type_name: "string" },  // variant: primary|secondary|ghost|danger
                PropertyContract { id: 3, type_name: "string" },  // icon
                PropertyContract { id: 4, type_name: "bool" },    // disabled
                PropertyContract { id: 5, type_name: "string" },  // class
            ],
            events: vec![1],  // on_click
            children: false,
        });

        // ====================================================================
        // FEEDBACK
        // ====================================================================

        // 50: Alert — alert message
        m.insert(50, ComponentContract {
            name: "alert",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "string" },  // message
                PropertyContract { id: 3, type_name: "string" },  // variant: info|success|warning|error
                PropertyContract { id: 4, type_name: "bool" },    // dismissible
            ],
            events: vec![1],  // on_dismiss
            children: false,
        });

        // 51: Toast — toast notification
        m.insert(51, ComponentContract {
            name: "toast",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // message
                PropertyContract { id: 2, type_name: "string" },  // variant: info|success|warning|error
                PropertyContract { id: 3, type_name: "int" },     // duration (ms)
            ],
            events: vec![1],
            children: false,
        });

        // 52: Modal — modal dialog
        m.insert(52, ComponentContract {
            name: "modal",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "bool" },    // open
                PropertyContract { id: 3, type_name: "string" },  // size: sm|md|lg|xl
            ],
            events: vec![1, 2],  // on_open, on_close
            children: true,
        });

        // 53: Tooltip — tooltip
        m.insert(53, ComponentContract {
            name: "tooltip",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // text
                PropertyContract { id: 2, type_name: "string" },  // position: top|bottom|left|right
            ],
            events: vec![],
            children: true,
        });

        // 54: Progress — progress bar
        m.insert(54, ComponentContract {
            name: "progress",
            properties: vec![
                PropertyContract { id: 1, type_name: "int" },     // value (0-100)
                PropertyContract { id: 2, type_name: "string" },  // variant: linear|circular
                PropertyContract { id: 3, type_name: "string" },  // color
            ],
            events: vec![],
            children: false,
        });

        // ====================================================================
        // DATA DISPLAY
        // ====================================================================

        // 60: Table — data table
        m.insert(60, ComponentContract {
            name: "table",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // headers (comma-separated)
                PropertyContract { id: 2, type_name: "string" },  // class
            ],
            events: vec![1],  // on_row_click
            children: true,
        });

        // 61: List — list display
        m.insert(61, ComponentContract {
            name: "list",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // class
                PropertyContract { id: 2, type_name: "string" },  // variant: default|card|inline
            ],
            events: vec![1],
            children: true,
        });

        // 62: Chart — chart display
        m.insert(62, ComponentContract {
            name: "chart",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // type: bar|line|pie|doughnut
                PropertyContract { id: 2, type_name: "string" },  // title
                PropertyContract { id: 3, type_name: "int" },     // width
                PropertyContract { id: 4, type_name: "int" },     // height
            ],
            events: vec![1],
            children: false,
        });

        // 63: Badge — badge/tag
        m.insert(63, ComponentContract {
            name: "badge",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // text
                PropertyContract { id: 2, type_name: "string" },  // variant: default|success|warning|error|info
            ],
            events: vec![],
            children: false,
        });

        // ====================================================================
        // DISPLAY
        // ====================================================================

        // 70: Avatar — user avatar
        m.insert(70, ComponentContract {
            name: "avatar",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // src
                PropertyContract { id: 2, type_name: "string" },  // name (initials fallback)
                PropertyContract { id: 3, type_name: "int" },     // size
                PropertyContract { id: 4, type_name: "string" },  // class
            ],
            events: vec![1],
            children: false,
        });

        // 71: Icon — icon display
        m.insert(71, ComponentContract {
            name: "icon",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // name
                PropertyContract { id: 2, type_name: "int" },     // size
                PropertyContract { id: 3, type_name: "string" },  // color
            ],
            events: vec![],
            children: false,
        });

        // 72: Card — generic card
        m.insert(72, ComponentContract {
            name: "card",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "string" },  // subtitle
                PropertyContract { id: 3, type_name: "string" },  // image
                PropertyContract { id: 4, type_name: "string" },  // class
            ],
            events: vec![1],
            children: true,
        });

        // 73: Accordion — collapsible section
        m.insert(73, ComponentContract {
            name: "accordion",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // title
                PropertyContract { id: 2, type_name: "bool" },    // open
            ],
            events: vec![1],  // on_toggle
            children: true,
        });

        // 74: Carousel — image/content carousel
        m.insert(74, ComponentContract {
            name: "carousel",
            properties: vec![
                PropertyContract { id: 1, type_name: "bool" },    // autoplay
                PropertyContract { id: 2, type_name: "int" },     // interval (ms)
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![1],  // on_slide
            children: true,
        });

        // ====================================================================
        // AGENT-SPECIFIC
        // ====================================================================

        // 80: Chat — chat interface
        m.insert(80, ComponentContract {
            name: "chat",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // agent name
                PropertyContract { id: 2, type_name: "string" },  // placeholder
                PropertyContract { id: 3, type_name: "string" },  // class
            ],
            events: vec![1],  // on_message
            children: true,
        });

        // 81: Message — chat message
        m.insert(81, ComponentContract {
            name: "message",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // sender
                PropertyContract { id: 2, type_name: "string" },  // content
                PropertyContract { id: 3, type_name: "string" },  // timestamp
                PropertyContract { id: 4, type_name: "string" },  // variant: user|agent|system
            ],
            events: vec![],
            children: false,
        });

        // 82: Typing Indicator
        m.insert(82, ComponentContract {
            name: "typing-indicator",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // agent name
                PropertyContract { id: 2, type_name: "bool" },    // active
            ],
            events: vec![],
            children: false,
        });

        // 83: Agent Card — agent info card
        m.insert(83, ComponentContract {
            name: "agent-card",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // name
                PropertyContract { id: 2, type_name: "string" },  // description
                PropertyContract { id: 3, type_name: "string" },  // avatar
                PropertyContract { id: 4, type_name: "string" },  // status: online|offline|busy
            ],
            events: vec![1],  // on_click
            children: false,
        });

        // 84: Tool Output — tool call result display
        m.insert(84, ComponentContract {
            name: "tool-output",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // tool name
                PropertyContract { id: 2, type_name: "string" },  // input
                PropertyContract { id: 3, type_name: "string" },  // output
                PropertyContract { id: 4, type_name: "string" },  // status: success|error|pending
            ],
            events: vec![],
            children: false,
        });

        // 85: Memory Display — memory record display
        m.insert(85, ComponentContract {
            name: "memory-display",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // key
                PropertyContract { id: 2, type_name: "string" },  // value
                PropertyContract { id: 3, type_name: "string" },  // scope
                PropertyContract { id: 4, type_name: "int" },     // confidence
            ],
            events: vec![],
            children: false,
        });

        // 86: Reasoning Trace — chain-of-thought display
        m.insert(86, ComponentContract {
            name: "reasoning-trace",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },  // instruction
                PropertyContract { id: 2, type_name: "string" },  // steps (JSON array)
                PropertyContract { id: 3, type_name: "string" },  // conclusion
            ],
            events: vec![],
            children: false,
        });

        m
    };
}

/// Total number of components
pub fn component_count() -> usize {
    COMPONENTS.len()
}
