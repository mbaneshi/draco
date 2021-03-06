use crate::{Mailbox, VNode, VText};
use derivative::Derivative;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::UnwrapThrowExt;
use web_sys as web;

pub trait Application: Sized + 'static {
    type Message;

    fn update(&mut self, _message: Self::Message, _mailbox: &Mailbox<Self::Message>) {}
    fn view(&self) -> VNode<Self::Message>;
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Instance<A: Application> {
    #[derivative(Debug = "ignore")]
    inner: Rc<Inner<A>>,
}

struct Inner<A: Application> {
    app: RefCell<A>,
    node: Cell<web::Node>,
    vnode: RefCell<VNode<A::Message>>,
    queue: RefCell<Vec<A::Message>>,
    is_updating: Cell<bool>,
    is_rendering: Cell<bool>,
}

impl<A: Application> Instance<A> {
    fn send(&self, message: A::Message) {
        self.push(message);
        self.update();
    }

    fn push(&self, message: A::Message) {
        self.inner.queue.borrow_mut().push(message);
    }

    fn update(&self) {
        if self.inner.is_rendering.get() {
            return;
        }

        // If we were called from inside the `while` loop below, bail out; the message will be
        // processed by the loop later.
        if self.inner.is_updating.get() {
            return;
        }

        self.inner.is_updating.replace(true);

        let mailbox = self.mailbox();

        while !self.inner.queue.borrow().is_empty() {
            let message = self.inner.queue.borrow_mut().remove(0);
            self.inner.app.borrow_mut().update(message, &mailbox);
        }

        self.inner.is_updating.replace(false);

        self.render();
    }

    fn render(&self) {
        self.inner.is_rendering.replace(true);
        let mut new_vnode = self.inner.app.borrow().view();
        let new_node = new_vnode.patch(&mut self.inner.vnode.borrow_mut(), &self.mailbox());
        self.inner.vnode.replace(new_vnode);
        self.inner.node.replace(new_node);
        self.inner.is_rendering.replace(false);
        if !self.inner.queue.borrow().is_empty() {
            self.update()
        }
    }

    fn mailbox(&self) -> Mailbox<A::Message> {
        let cloned = self.clone();
        Mailbox::new(move |message| {
            cloned.send(message);
        })
    }
}

impl<A: Application> std::clone::Clone for Instance<A> {
    fn clone(&self) -> Self {
        Instance {
            inner: Rc::clone(&self.inner),
        }
    }
}

pub fn start<A: Application>(app: A, node: web::Node) -> Mailbox<A::Message> {
    let mut vnode = VText::new("!");
    let new_node = vnode.create().into();
    node.parent_node()
        .unwrap_throw()
        .replace_child(&new_node, &node)
        .unwrap_throw();
    let instance = Instance {
        inner: Rc::new(Inner {
            app: RefCell::new(app),
            node: Cell::new(new_node),
            vnode: RefCell::new(vnode.into()),
            is_updating: Cell::new(false),
            is_rendering: Cell::new(false),
            queue: RefCell::new(Vec::new()),
        }),
    };
    instance.render();
    instance.mailbox()
}
