use std::{
    any::{type_name, Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

#[derive(Default)]
pub struct App {
    resources: HashMap<TypeId, Rc<dyn Any>>,
    update_systems: Vec<Rc<dyn System>>,
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
        S: Fn<I, Output = ()> + 'static,
        I: std::marker::Tuple + 'static,
        SystemWrapper<S, I>: System,
    {
        self.update_systems.push(Rc::new(SystemWrapper {
            system,
            _pd: PhantomData,
        }))
    }

    pub fn execute(&self) {
        for system in &self.update_systems {
            system.execute(&self)
        }
    }
}

pub trait SystemParameter {
    type BorrowedFromApp;

    fn get_from_world(app: &App) -> Self::BorrowedFromApp;
}

pub struct Res<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> Clone for Res<T> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone(),
        }
    }
}

impl<T> Res<T> {
    pub fn get(&self) -> Ref<'_, T> {
        self.rc
            .try_borrow()
            .expect(&format!("borrow error resource: '{}'", type_name::<T>()))
    }
}

impl<T> SystemParameter for Res<T>
where
    T: 'static,
{
    type BorrowedFromApp = Res<T>;

    fn get_from_world(app: &App) -> Self::BorrowedFromApp {
        app.get_resource::<T>().expect(&format!(
            "failed getting resource: '{}' from world",
            type_name::<T>()
        ))
    }
}

pub struct ResMut<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> Clone for ResMut<T> {
    fn clone(&self) -> Self {
        Self {
            rc: self.rc.clone(),
        }
    }
}

impl<T> ResMut<T> {
    pub fn get(&self) -> Ref<'_, T> {
        self.rc
            .try_borrow()
            .expect(&format!("borrow error resource: '{}'", type_name::<T>()))
    }

    pub fn get_mut(&self) -> RefMut<'_, T> {
        self.rc
            .try_borrow_mut()
            .expect(&format!("borrow error resource: '{}'", type_name::<T>()))
    }

    pub fn replace(&self, value: T) -> T {
        self.rc.replace(value)
    }
}

impl<T> SystemParameter for ResMut<T>
where
    T: 'static,
{
    type BorrowedFromApp = ResMut<T>;

    fn get_from_world(app: &App) -> Self::BorrowedFromApp {
        app.get_resource_mut::<T>().expect(&format!(
            "failed getting resource {} from world",
            type_name::<T>()
        ))
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
impl_system_for_system_wrappers!(A B C D E);
impl_system_for_system_wrappers!(A B C D E F);
impl_system_for_system_wrappers!(A B C D E F G);
impl_system_for_system_wrappers!(A B C D E F G H);
impl_system_for_system_wrappers!(A B C D E F G H I);
