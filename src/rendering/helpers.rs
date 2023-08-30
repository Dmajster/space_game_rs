use std::marker::PhantomData;

#[derive(Debug)]
pub struct Handle<T> {
    pub index: usize,
    pub generation: usize,
    _pd: PhantomData<T>,
}

impl<T> Handle<T> {
    pub const EMPTY: Handle<T> = Handle {
        index: usize::MAX,
        generation: usize::MAX,
        _pd: PhantomData,
    };
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Handle::EMPTY
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            generation: self.generation.clone(),
            _pd: self._pd.clone(),
        }
    }
}

impl<T> Copy for Handle<T> {}

#[derive(Debug)]
pub struct Pool<T> {
    objects: Vec<T>,
    generations: Vec<usize>,
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self {
            objects: Default::default(),
            generations: Default::default(),
        }
    }
}

impl<T> Pool<T> {
    pub fn add(&mut self, object: T) -> Handle<T> {
        let index = self.objects.len();
        self.objects.push(object);
        self.generations.push(0);

        Handle {
            index,
            generation: 0,
            _pd: PhantomData,
        }
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        if handle.index < self.objects.len() && handle.generation == self.generations[handle.index]
        {
            Some(&self.objects[handle.index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        if handle.index < self.objects.len() && handle.generation == self.generations[handle.index]
        {
            Some(&mut self.objects[handle.index])
        } else {
            None
        }
    }
}
