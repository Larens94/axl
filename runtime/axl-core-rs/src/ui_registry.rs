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

pub fn annotation_kind_valid(kind: i32) -> bool {
    matches!(kind, 1 | 2 | 3)
}

lazy_static::lazy_static! {
    static ref COMPONENTS: HashMap<i32, ComponentContract> = {
        let mut m = HashMap::new();
        m.insert(1, ComponentContract {
            name: "app",
            properties: vec![PropertyContract { id: 1, type_name: "string" }],
            events: vec![],
            children: true,
        });
        m.insert(2, ComponentContract {
            name: "hero",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },
                PropertyContract { id: 2, type_name: "string" },
                PropertyContract { id: 3, type_name: "string" },
                PropertyContract { id: 4, type_name: "string" },
                PropertyContract { id: 5, type_name: "string" },
            ],
            events: vec![1, 2],
            children: false,
        });
        m.insert(3, ComponentContract {
            name: "shelf",
            properties: vec![PropertyContract { id: 1, type_name: "string" }],
            events: vec![],
            children: true,
        });
        m.insert(4, ComponentContract {
            name: "media-card",
            properties: vec![
                PropertyContract { id: 1, type_name: "string" },
                PropertyContract { id: 2, type_name: "string" },
                PropertyContract { id: 3, type_name: "int" },
                PropertyContract { id: 4, type_name: "int" },
            ],
            events: vec![1],
            children: false,
        });
        m
    };
}
