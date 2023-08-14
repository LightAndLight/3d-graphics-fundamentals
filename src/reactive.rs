pub struct Var<A> {
    changed: bool,
    value: A,
}

impl<A> Var<A> {
    pub fn new(value: A) -> Self {
        Self {
            changed: false,
            value,
        }
    }

    pub fn get(&self) -> &A {
        &self.value
    }

    pub fn set(&mut self, value: A) {
        self.value = value;
        self.changed = true;
    }

    pub fn modify(&mut self, f: &mut dyn FnMut(&A) -> A) {
        self.value = f(&self.value);
        self.changed = true;
    }

    pub fn modify_mut(&mut self, f: &mut dyn FnMut(&mut A)) {
        f(&mut self.value);
        self.changed = true;
    }

    pub fn as_components(&mut self) -> (&mut A, &mut bool) {
        (&mut self.value, &mut self.changed)
    }

    pub fn react(&mut self, on_change: &mut dyn FnMut(&A)) {
        if self.changed {
            on_change(&self.value);
            self.changed = false;
        }
    }
}
