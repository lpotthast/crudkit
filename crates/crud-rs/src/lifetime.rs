use async_trait::async_trait;

use crate::resource::CrudResource;

pub enum Abort {
    Yes {
        reason: String,
    },
    No
}

#[async_trait]
pub trait CrudLifetime<R: CrudResource> {
    async fn before_create(
        create_model: &R::CreateModel,
        active_model: &mut R::ActiveModel,
        context: &R::Context,
        mut data: R::HookData,
    ) -> Result<(Abort, R::HookData), String>;

    async fn after_create(
        create_model: &R::CreateModel,
        model: &R::Model,
        context: &R::Context,
        mut data: R::HookData,
    ) -> Result<R::HookData, String>;

    async fn before_update(
        update_model: &R::UpdateModel,
        active_model: &mut R::ActiveModel,
        context: &R::Context,
        mut data: R::HookData,
    ) -> Result<(Abort, R::HookData), String>;

    async fn after_update(
        update_model: &R::UpdateModel,
        model: &R::Model,
        context: &R::Context,
        mut data: R::HookData,
    ) -> Result<R::HookData, String>;

    async fn before_delete(
        model: &R::Model,
        context: &R::Context,
        mut data: R::HookData,
    ) -> Result<(Abort, R::HookData), String>;

    async fn after_delete(
        model: &R::Model,
        context: &R::Context,
        mut data: R::HookData,
    ) -> Result<R::HookData, String>;
}

pub struct NoopLifetimeHooks {}

#[async_trait]
impl<R: CrudResource> CrudLifetime<R> for NoopLifetimeHooks {
    async fn before_create(
        _create_model: &<R as CrudResource>::CreateModel,
        _active_model: &mut <R as CrudResource>::ActiveModel,
        _context: &<R as CrudResource>::Context,
        data: <R as CrudResource>::HookData,
    ) -> Result<(Abort, <R as CrudResource>::HookData), String> {
        Ok((Abort::No, data))
    }

    async fn after_create(
        _create_model: &<R as CrudResource>::CreateModel,
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        data: <R as CrudResource>::HookData,
    ) -> Result<<R as CrudResource>::HookData, String> {
        Ok(data)
    }

    async fn before_update(
        _update_model: &<R as CrudResource>::UpdateModel,
        _active_model: &mut <R as CrudResource>::ActiveModel,
        _context: &<R as CrudResource>::Context,
        data: <R as CrudResource>::HookData,
    ) -> Result<(Abort, <R as CrudResource>::HookData), String> {
        Ok((Abort::No, data))
    }

    async fn after_update(
        _update_model: &<R as CrudResource>::UpdateModel,
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        data: <R as CrudResource>::HookData,
    ) -> Result<<R as CrudResource>::HookData, String> {
        Ok(data)
    }

    async fn before_delete(
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        data: <R as CrudResource>::HookData,
    ) -> Result<(Abort, <R as CrudResource>::HookData), String> {
        Ok((Abort::No, data))
    }

    async fn after_delete(
        _model: &<R as CrudResource>::Model,
        _context: &<R as CrudResource>::Context,
        data: <R as CrudResource>::HookData,
    ) -> Result<<R as CrudResource>::HookData, String> {
        Ok(data)
    }
}
