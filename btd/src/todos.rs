pub struct TodoList {
    items: Vec<String>,
}

impl TodoList {
    pub fn new() -> Self {
        TodoList { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }

    pub fn get_items(&self) -> &Vec<String> {
        &self.items
    }
}

#[derive(PartialEq, Clone)]
pub struct TodoItem {
    id: usize,
    pub description: String,
    pub completed: bool,
}

impl TodoItem {
    pub fn new(id: usize, description: String) -> Self {
        TodoItem {
            id,
            description,
            completed: false,
        }
    }

    pub fn mark_completed(&mut self) {
        println!("Marking item as completed");
        self.completed = true;
    }

    pub fn mark_incomplete(&mut self) {
        println!("Marking item as incomplete");
        self.completed = false;
    }
}
