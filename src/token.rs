use std::marker::PhantomData;
use mould::prelude::*;
use super::Role;

pub trait Manager<R: Role> {
    fn set_role(&mut self, token: &str) -> Result<bool, &str>;
    fn acquire_token(&mut self) -> Result<String, &str>;
    fn drop_token(&mut self) -> Result<(), &str>;
}

pub enum Permission {
    CanAuth,
    CanAcquire,
}

impl Rights for Permission { }

/// A handler which use `TokenManager` to set role to session.
/// The following actions available:
/// * `do-auth` - try to authorize by token
pub struct TokenService<R> {
    _role: PhantomData<R>,
}

unsafe impl<R> Sync for TokenService<R> { }
unsafe impl<R> Send for TokenService<R> { }

impl<R> TokenService<R> {
    pub fn new() -> Self {
        TokenService {
            _role: PhantomData,
        }
    }
}

impl<T, R> service::Service<T> for TokenService<R>
    where T: Session + Require<Permission> + Manager<R>, R: Role,
{
    fn route(&self, action: &str) -> service::Result<service::Action<T>> {
        match action {
            "do-login" => Ok(do_login::action()),
            "acquire-new" => Ok(acquire_new::action()),
            "drop-token" => Ok(drop_token::action()),
            _ => Err(service::ErrorKind::ActionNotFound.into()),
        }
    }
}

mod do_login {
    use super::*;

    pub fn action<T, R>() -> service::Action<T>
        where T: Session + Require<Permission> + Manager<R>, R: Role,
    {
        let worker = Worker {
            _role: PhantomData,
        };
        service::Action::from_worker(worker)
    }

    struct Worker<R> {
        _role: PhantomData<R>,
    }

    #[derive(Deserialize)]
    struct Request {
        token: String,
    }

    #[derive(Serialize)]
    struct Out {
        success: bool,
    }

    impl<T, R> worker::Worker<T> for Worker<R>
        where T: Session + Require<Permission> + Manager<R>, R: Role,
    {
        type Request = Request;
        type In = ();
        type Out = Out;

        fn prepare(&mut self, session: &mut T, request: Self::Request) -> worker::Result<Shortcut<Self::Out>> {
            session.require(&Permission::CanAuth)?;
            let success = session.set_role(&request.token)?;
            Ok(Shortcut::OneItemAndDone(Out { success }))
        }
    }
}

mod acquire_new {
    use super::*;

    pub fn action<T, R>() -> service::Action<T>
        where T: Session + Require<Permission> + Manager<R>, R: Role,
    {
        let worker = Worker {
            _role: PhantomData,
        };
        service::Action::from_worker(worker)
    }

    struct Worker<R> {
        _role: PhantomData<R>,
    }

    #[derive(Serialize)]
    struct Out {
        token: String,
    }

    impl<T, R> worker::Worker<T> for Worker<R>
        where T: Session + Require<Permission> + Manager<R>, R: Role,
    {
        type Request = ();
        type In = ();
        type Out = Out;

        fn prepare(&mut self, session: &mut T, _: Self::Request) -> worker::Result<Shortcut<Self::Out>> {
            session.require(&Permission::CanAcquire)?;
            let token = session.acquire_token()?;
            Ok(Shortcut::OneItemAndDone(Out { token }))
        }
    }

}
mod drop_token {
    use super::*;

    pub fn action<T, R>() -> service::Action<T>
        where T: Session + Require<Permission> + Manager<R>, R: Role,
    {
        let worker = Worker {
            _role: PhantomData,
        };
        service::Action::from_worker(worker)
    }

    struct Worker<R> {
        _role: PhantomData<R>,
    }

    impl<T, R> worker::Worker<T> for Worker<R>
        where T: Session + Require<Permission> + Manager<R>, R: Role,
    {
        type Request = ();
        type In = ();
        type Out = ();

        fn prepare(&mut self, session: &mut T, _: Self::Request) -> worker::Result<Shortcut<Self::Out>> {
            session.require(&Permission::CanAcquire)?;
            session.drop_token()?;
            Ok(Shortcut::Done)
        }
    }

}
