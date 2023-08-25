use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

#[derive(Default)]
pub struct App {
    resources: HashMap<TypeId, Rc<dyn Any>>,
    systems: Vec<Rc<dyn System>>,
}

impl App {
    pub fn add_resource<R>(&mut self, resource: R)
    where
        R: 'static,
    {
        self.resources
            .insert(TypeId::of::<R>(), Rc::new(RefCell::new(resource)));
    }

    pub fn get_resource<R>(&self) -> Option<Res<R>>
    where
        R: 'static,
    {
        Some(Res {
            rc: self
                .resources
                .get(&TypeId::of::<R>())?
                .clone()
                .downcast::<RefCell<R>>()
                .unwrap(),
        })
    }

    pub fn get_resource_mut<R>(&self) -> Option<ResMut<R>>
    where
        R: 'static,
    {
        Some(ResMut {
            rc: self
                .resources
                .get(&TypeId::of::<R>())?
                .clone()
                .downcast::<RefCell<R>>()
                .unwrap(),
        })
    }

    pub fn add_system<S, I>(&mut self, system: S)
    where
        S: Fn<(I,), Output = ()> + 'static,
        I: 'static,
        SystemWrapper<S, (I,)>: System,
    {
        self.systems.push(Rc::new(SystemWrapper {
            system,
            _pd: PhantomData,
        }))
    }

    pub fn execute(&self) {
        for system in &self.systems {
            system.execute(&self)
        }
    }
}

// #[derive(Debug)]
// pub struct EventQueue<T> {
//     events: Vec<T>,
// }

// impl<T> EventQueue<T> {
//     pub fn push_event(&mut self, event: T) {
//         self.events.push(event);
//     }
// }

// impl<T> Default for EventQueue<T> {
//     fn default() -> Self {
//         Self { events: vec![] }
//     }
// }

pub trait SystemParameter {
    type BorrowedFromApp;

    fn get_from_world(app: &App) -> Self::BorrowedFromApp;
}

pub struct Res<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> Res<T> {
    pub fn get(&self) -> Ref<'_, T> {
        self.rc.borrow()
    }
}

// impl<T> Debug for Resource<T>
// where
//     T: Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Resource").field("rc", &self.rc).finish()
//     }
// }

impl<T> SystemParameter for Res<T>
where
    T: 'static,
{
    type BorrowedFromApp = Res<T>;

    fn get_from_world(app: &App) -> Self::BorrowedFromApp {
        app.get_resource::<T>().unwrap()
    }
}

pub struct ResMut<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> ResMut<T> {
    pub fn get_mut(&self) -> RefMut<'_, T> {
        self.rc.borrow_mut()
    }
}

// impl<T> Debug for ResourceMut<T>
// where
//     T: Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Resource").field("rc", &self.rc).finish()
//     }
// }

impl<T> SystemParameter for ResMut<T>
where
    T: 'static,
{
    type BorrowedFromApp = ResMut<T>;

    fn get_from_world(app: &App) -> Self::BorrowedFromApp {
        app.get_resource_mut::<T>().unwrap()
    }
}

pub struct SystemWrapper<S, I>
where
    S: Fn<I, Output = ()>,
    I: std::marker::Tuple,
{
    system: S,
    _pd: PhantomData<I>,
}

pub trait System {
    fn execute(&self, app: &App);
}

macro_rules! impl_system_for_system_wrappers {
    ($($T:ident)+) => {
        impl<S, $($T,)+> System for SystemWrapper<S, ($($T,)+)>
        where
            S: Fn<($($T,)+), Output = ()>,
            $($T: SystemParameter<BorrowedFromApp = $T> + 'static,)+
        {
            fn execute(&self, app: &App) {
                self.system.call(($($T::get_from_world(app),)+));
            }
        }
    };
}

impl_system_for_system_wrappers!(A);
impl_system_for_system_wrappers!(A B);
impl_system_for_system_wrappers!(A B C);
impl_system_for_system_wrappers!(A B C D);
