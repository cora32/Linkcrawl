use std::fmt;
use std::sync::RwLock;

lazy_static! {
        pub static ref MUTEX_ID_COUNTER:RwLock<u32>= RwLock::new(0);
    }

#[derive(Debug)]
pub struct LinkTreeNode {
    id: u32,
    link: String,
    node_list: Vec<LinkTreeNode>,
    parent_id: u32
}

impl fmt::Display for LinkTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decimals = f.width().unwrap_or(0);
        let mut string = format!("{: <width$}{}; id:{}; parent_id:{};\n", " ",
                                 self.link, self.id, self.parent_id, width = decimals);

        for x in &self.node_list {
            string.push_str(&format!("{:<1$}", x, decimals + 10))
        }

        write!(f, "{}", string)
    }
}

impl LinkTreeNode {
    pub fn create(link: &String) -> LinkTreeNode {
        let id = MUTEX_ID_COUNTER.try_read().unwrap().to_owned();
        let result = LinkTreeNode {
            id,
            link: link.clone(),
            node_list: vec![],
            parent_id: 0,
        };
        *MUTEX_ID_COUNTER.write().unwrap() = id + 1;
        result
    }

    pub fn add_child(&mut self, mut node: LinkTreeNode) {
        node.set_parent_id(self.id);
        self.node_list.push(node);
    }

    pub fn set_parent_id(&mut self, parent_id: u32) {
        self.parent_id = parent_id;
    }

    pub fn link(&mut self) -> &String {
        &self.link
    }

    pub fn node_list(&mut self) -> &mut Vec<LinkTreeNode>{
        &mut self.node_list
    }
}