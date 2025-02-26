use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

fn main() {
    // println!("Hello, world!");
    let gc = TracingGC::new();

    let obj1 = gc.allocate(Value {
        value: 1,
        next: None,
    });

    let obj2 = gc.allocate(Value {
        value: 2,
        next: Some(obj1.clone()),
    });

    gc.add_root(obj2.id());
    gc.collect();
}

pub struct TracingGC {
    objects: RefCell<HashMap<usize, (Rc<dyn GcObject>, Cell<bool>)>>,
    roots: RefCell<HashSet<usize>>,
    next_id: Cell<usize>,
}

pub trait GcObject {
    fn trace(&self, tracer: &TracingGC);
}

impl TracingGC {
    pub fn new() -> TracingGC {
        TracingGC {
            objects: RefCell::new(HashMap::new()),
            roots: RefCell::new(HashSet::new()),
            next_id: Cell::new(0),
        }
    }

    pub fn allocate<T: GcObject + 'static>(&self, value: T) -> GcRef<T> {
        let id = self.next_id.get();
        self.next_id.set(id + 1);

        let rc = Rc::new(value);
        let address = Rc::as_ptr(&rc) as *const ();

        println!(
            "
          | allocated | idx: {} | address: {:p} | marked: false |
        ",
            id, address
        );

        self.objects
            .borrow_mut()
            .insert(id, (rc.clone(), Cell::new(false)));

        GcRef { id, rc }
    }

    pub fn add_root(&self, id: usize) {
        self.roots.borrow_mut().insert(id);
        println!("| root (add) | id: {} |", id);
    }

    pub fn remove_root(&self, id: usize) {
        self.roots.borrow_mut().remove(&id);
        println!("| root (remove) | id: {} |", id);
    }

    fn mark(&self, obj: &Rc<dyn GcObject>, marked: &Cell<bool>) {
        if marked.get() {
            return;
        }

        marked.set(true);

        println!(
            "| marked      | address: {:p} |",
            Rc::as_ptr(obj) as *const ()
        );

        obj.trace(self);
    }

    pub fn collect(&self) {
        for (_, (_, marked)) in self.objects.borrow().iter() {
            marked.set(false)
        }

        println!("\n| mark phase |");
        for root in self.roots.borrow().iter() {
            if let Some((rc, marked)) = self.objects.borrow().get(root) {
                marked.set(true);
            }
        }

        println!("\n| sweep phase |");
        let init_count = self.objects.borrow().len();
        self.objects.borrow_mut().retain(|&id, (rc, marked)| {
            if marked.get() {
                let address = Rc::as_ptr(&rc) as *const ();
                println!("| live object | id: {} | address: {:p} |", id, address);
                true
            } else {
                let address = Rc::as_ptr(rc) as *const ();
                println!("| collected   | id: {} | address: {:p} |", id, address);
                false
            }
        });

        let collected_count = init_count - self.objects.borrow().len();
        println!(
            "| collection complete | total collected: {} | remaining: {} |\n",
            collected_count,
            self.objects.borrow().len()
        );
    }
}

pub struct GcRef<T: GcObject> {
    id: usize,
    rc: Rc<T>,
}

impl<T: GcObject> Clone for GcRef<T> {
    fn clone(&self) -> Self {
        GcRef {
            id: self.id,
            rc: self.rc.clone(),
        }
    }
}

impl<T: GcObject> GcRef<T> {
    pub fn id(&self) -> usize {
        self.id
    }
}

impl GcObject for TracingGC {
    fn trace(&self, tracer: &TracingGC) {
        for root in self.roots.borrow().iter() {
            if let Some((obj, marked)) = self.objects.borrow().get(root) {
                tracer.mark(obj, marked);
            }
        }
    }
}

struct Value {
    value: i32,
    next: Option<GcRef<Value>>,
}

impl GcObject for Value {
    fn trace(&self, tracer: &TracingGC) {
        if let Some(value) = &self.next {
            if let Some((rc, marked)) = tracer.objects.borrow_mut().get(&value.id) {
                tracer.mark(rc, marked);
            }
        }
    }
}
