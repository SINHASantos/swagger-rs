//! Module for API context management.

use hyper;
use auth::{Authorization, AuthData};
use std::marker::Sized;
extern crate slog;

/// Request context, both as received in a server handler or as sent in a
/// client request. When REST microservices are chained, the Context passes
/// data from the server API to any further HTTP requests.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// Tracking ID when passing a request to another microservice.
    pub x_span_id: XSpanIdString,

    /// Authorization data, filled in from middlewares.
    pub authorization: Option<Authorization>,
    /// Raw authentication data, for use in making HTTP requests as a client.
    pub auth_data: Option<AuthData>,
    logger: Option<slog::Logger>,
}

#[derive(Debug, Clone, Default)]
pub struct XSpanIdString(pub String);

pub trait Has<T> {
    fn set(&mut self, T);
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

pub trait ExtendsWith<C, T>: Has<T> {
    fn new(inner: C, item: T) -> Self;
}

pub struct ContextExtension<C, T> {
    inner: C,
    item: T,
}

impl<C, T> Has<T> for ContextExtension<C, T> {
    fn set(&mut self, item: T) {
        self.item = item;
    }

    fn get(&self) -> &T {
        &self.item
    }

    fn get_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<C, T> ExtendsWith<C, T> for ContextExtension<C, T> {
    fn new(inner: C, item: T) -> Self {
        ContextExtension { inner, item }
    }
}

impl<C: Has<XSpanIdString>> Has<XSpanIdString> for ContextExtension<C, Option<Authorization>> {
    fn set(&mut self, item: XSpanIdString) {
        self.inner.set(item);
    }

    fn get(&self) -> &XSpanIdString {
        self.inner.get()
    }

    fn get_mut(&mut self) -> &mut XSpanIdString {
        self.inner.get_mut()
    }
}

impl<C: Has<Option<Authorization>>> Has<Option<Authorization>> for ContextExtension<C, XSpanIdString> {
    fn set(&mut self, item: Option<Authorization>) {
        self.inner.set(item);
    }

    fn get(&self) -> &Option<Authorization> {
        self.inner.get()
    }

    fn get_mut(&mut self) -> &mut Option<Authorization> {
        self.inner.get_mut()
    }
}

impl<C: Has<XSpanIdString>> Has<XSpanIdString> for ContextExtension<C, AuthData> {
    fn set(&mut self, item: XSpanIdString) {
        self.inner.set(item);
    }

    fn get(&self) -> &XSpanIdString {
        self.inner.get()
    }

    fn get_mut(&mut self) -> &mut XSpanIdString {
        self.inner.get_mut()
    }
}

impl<C: Has<AuthData>> Has<AuthData> for ContextExtension<C, XSpanIdString> {
    fn set(&mut self, item: AuthData) {
        self.inner.set(item);
    }

    fn get(&self) -> &AuthData {
        self.inner.get()
    }

    fn get_mut(&mut self) -> &mut AuthData {
        self.inner.get_mut()
    }
}

impl<C: Has<AuthData>> Has<AuthData> for ContextExtension<C, Option<Authorization>> {
    fn set(&mut self, item: AuthData) {
        self.inner.set(item);
    }

    fn get(&self) -> &AuthData {
        self.inner.get()
    }

    fn get_mut(&mut self) -> &mut AuthData {
        self.inner.get_mut()
    }
}

impl<C: Has<Option<Authorization>>> Has<Option<Authorization>> for ContextExtension<C, AuthData> {
    fn set(&mut self, item: Option<Authorization>) {
        self.inner.set(item);
    }

    fn get(&self) -> &Option<Authorization> {
        self.inner.get()
    }

    fn get_mut(&mut self) -> &mut Option<Authorization> {
        self.inner.get_mut()
    }
}

/// Trait for retrieving a logger from a struct.
pub trait HasLogger {
    /// Retrieve the context logger
    fn logger(&self) -> &Option<slog::Logger>;

    /// Set the context logger
    fn set_logger(&mut self, logger: slog::Logger);
}

impl HasLogger for Context {
    fn logger(&self) -> &Option<slog::Logger> {
        &self.logger
    }

    fn set_logger(&mut self, logger: slog::Logger) {
        self.logger = Some(logger);
    }
}

impl Has<XSpanIdString> for Context {
    fn set(&mut self, item: XSpanIdString) {
        self.x_span_id = item;
    }

    fn get(&self) -> &XSpanIdString{
        &self.x_span_id
    }

    fn get_mut(&mut self) -> &mut XSpanIdString {
        &mut self.x_span_id
    }
}

impl Context {
    /// Create a new, empty, `Context`.
    pub fn new() -> Context {
        Context::default()
    }

    /// Create a `Context` with a given span ID.
    pub fn new_with_span_id<S: Into<String>>(x_span_id: S) -> Context {
        Context {
            x_span_id: XSpanIdString(x_span_id.into()),
            ..Context::default()
        }
    }

    /// Set Basic authentication
    pub fn auth_basic(&mut self, username: &str, password: &str) {
        self.auth_data = Some(AuthData::Basic(hyper::header::Basic {
            username: username.to_owned(),
            password: Some(password.to_owned()),
        }));
    }

    /// Set Bearer token authentication
    pub fn auth_bearer(&mut self, token: &str) {
        self.auth_data = Some(AuthData::Bearer(
            hyper::header::Bearer { token: token.to_owned() },
        ));
    }

    /// Set ApiKey authentication
    pub fn auth_apikey(&mut self, apikey: &str) {
        self.auth_data = Some(AuthData::ApiKey(apikey.to_owned()));
    }
}

/// Context wrapper, to bind an API with a context.
#[derive(Debug)]
pub struct ContextWrapper<'a, T: 'a, C> {
    api: &'a T,
    context: C,
}

impl<'a, T, C> ContextWrapper<'a, T, C> {
    /// Create a new ContextWrapper, binding the API and context.
    pub fn new(api: &'a T, context: C) -> ContextWrapper<'a, T, C> {
        ContextWrapper { api, context }
    }

    /// Borrows the API.
    pub fn api(&self) -> &T {
        self.api
    }

    /// Borrows the context.
    pub fn context(&self) -> &C {
        &self.context
    }
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<'a, C>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self: &'a Self, context: C) -> ContextWrapper<'a, Self, C> {
        ContextWrapper::<Self, C>::new(self, context)
    }
}
