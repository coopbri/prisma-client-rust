use std::marker::PhantomData;

use prisma_models::PrismaValue;
use query_core::{Operation, QueryValue, Selection, SelectionBuilder};
use serde::de::DeserializeOwned;

use crate::{
    merged_object,
    select::{Select, SelectType},
    BatchQuery,
};

use super::{
    count::Count, delete_many::DeleteMany, QueryContext, QueryInfo, SerializedWhere, UpdateMany,
};

pub struct FindMany<'a, Where, With, OrderBy, Cursor, Set, Data>
where
    Where: Into<SerializedWhere>,
    With: Into<Selection>,
    OrderBy: Into<(String, PrismaValue)>,
    Cursor: Into<(String, PrismaValue)>,
    Set: Into<(String, PrismaValue)>,
    Data: DeserializeOwned,
{
    ctx: QueryContext<'a>,
    info: QueryInfo,
    pub where_params: Vec<Where>,
    pub with_params: Vec<With>,
    pub order_by_params: Vec<OrderBy>,
    pub cursor_params: Vec<Cursor>,
    pub skip: Option<i64>,
    pub take: Option<i64>,
    _data: PhantomData<(Set, Data)>,
}

impl<'a, Where, With, OrderBy, Cursor, Set, Data>
    FindMany<'a, Where, With, OrderBy, Cursor, Set, Data>
where
    Where: Into<SerializedWhere>,
    With: Into<Selection>,
    OrderBy: Into<(String, PrismaValue)>,
    Cursor: Into<(String, PrismaValue)>,
    Set: Into<(String, PrismaValue)>,
    Data: DeserializeOwned,
{
    pub fn new(ctx: QueryContext<'a>, info: QueryInfo, where_params: Vec<Where>) -> Self {
        Self {
            ctx,
            info,
            where_params,
            with_params: vec![],
            order_by_params: vec![],
            cursor_params: vec![],
            skip: None,
            take: None,
            _data: PhantomData,
        }
    }

    pub fn with(mut self, param: impl Into<With>) -> Self {
        self.with_params.push(param.into());
        self
    }

    pub fn order_by(mut self, param: impl Into<OrderBy>) -> Self {
        self.order_by_params.push(param.into());
        self
    }

    pub fn cursor(mut self, param: impl Into<Cursor>) -> Self {
        self.cursor_params.push(param.into());
        self
    }

    pub fn skip(mut self, skip: i64) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn take(mut self, take: i64) -> Self {
        self.take = Some(take);
        self
    }

    pub fn update(self, data: Vec<Set>) -> UpdateMany<'a, Where, Set> {
        let Self {
            ctx,
            info,
            where_params,
            ..
        } = self;

        UpdateMany::new(ctx, info, where_params, data)
    }

    pub fn delete(self) -> DeleteMany<'a, Where> {
        let Self {
            ctx,
            info,
            where_params,
            ..
        } = self;

        DeleteMany::new(ctx, info, where_params)
    }

    pub fn count(self) -> Count<'a, Where, OrderBy, Cursor> {
        let Self {
            ctx,
            info,
            where_params,
            ..
        } = self;

        Count::new(ctx, info, where_params)
    }

    fn to_selection(
        model: &str,
        where_params: Vec<Where>,
        order_by_params: Vec<OrderBy>,
        cursor_params: Vec<Cursor>,
        skip: Option<i64>,
        take: Option<i64>,
    ) -> SelectionBuilder {
        let mut selection = Selection::builder(format!("findMany{}", model));

        selection.alias("result");

        if where_params.len() > 0 {
            selection.push_argument(
                "where",
                merged_object(
                    where_params
                        .into_iter()
                        .map(Into::<SerializedWhere>::into)
                        .map(|s| (s.field, s.value.into()))
                        .collect(),
                ),
            );
        }

        if order_by_params.len() > 0 {
            selection.push_argument(
                "orderBy".to_string(),
                PrismaValue::Object(order_by_params.into_iter().map(Into::into).collect()),
            );
        }

        if cursor_params.len() > 0 {
            selection.push_argument(
                "cursor".to_string(),
                PrismaValue::Object(cursor_params.into_iter().map(Into::into).collect()),
            );
        }

        skip.map(|skip| selection.push_argument("skip".to_string(), PrismaValue::Int(skip as i64)));
        take.map(|take| selection.push_argument("take".to_string(), PrismaValue::Int(take as i64)));

        selection
    }

    pub fn select<S: SelectType<Data>>(self, select: S) -> Select<'a, Vec<S::Data>> {
        let mut selection = Self::to_selection(
            self.info.model,
            self.where_params,
            self.order_by_params,
            self.cursor_params,
            self.skip,
            self.take,
        );

        selection.nested_selections(select.to_selections());

        let op = Operation::Read(selection.build());

        Select::new(self.ctx, op)
    }

    pub(crate) fn exec_operation(self) -> (Operation, QueryContext<'a>) {
        let QueryInfo {
            model,
            mut scalar_selections,
        } = self.info;

        let mut selection = Self::to_selection(
            model,
            self.where_params,
            self.order_by_params,
            self.cursor_params,
            self.skip,
            self.take,
        );

        if self.with_params.len() > 0 {
            scalar_selections.append(&mut self.with_params.into_iter().map(Into::into).collect());
        }
        selection.nested_selections(scalar_selections);

        (Operation::Read(selection.build()), self.ctx)
    }

    pub async fn exec(self) -> super::Result<Vec<Data>> {
        let (op, ctx) = self.exec_operation();

        ctx.execute(op).await
    }
}

impl<'a, Where, With, OrderBy, Cursor, Set, Data> BatchQuery
    for FindMany<'a, Where, With, OrderBy, Cursor, Set, Data>
where
    Where: Into<SerializedWhere>,
    With: Into<Selection>,
    OrderBy: Into<(String, PrismaValue)>,
    Cursor: Into<(String, PrismaValue)>,
    Set: Into<(String, PrismaValue)>,
    Data: DeserializeOwned,
{
    type RawType = Data;
    type ReturnType = Self::RawType;

    fn graphql(self) -> Operation {
        self.exec_operation().0
    }

    fn convert(raw: super::Result<Self::RawType>) -> super::Result<Self::ReturnType> {
        raw
    }
}

#[derive(Clone)]
pub struct ManyArgs<Where, With, OrderBy, Cursor>
where
    Where: Into<SerializedWhere>,
    With: Into<Selection>,
    OrderBy: Into<(String, PrismaValue)>,
    Cursor: Into<(String, PrismaValue)>,
{
    pub where_params: Vec<Where>,
    pub with_params: Vec<With>,
    pub order_by_params: Vec<OrderBy>,
    pub cursor_params: Vec<Cursor>,
    pub skip: Option<i64>,
    pub take: Option<i64>,
}

impl<Where, With, OrderBy, Cursor> ManyArgs<Where, With, OrderBy, Cursor>
where
    Where: Into<SerializedWhere>,
    With: Into<Selection>,
    OrderBy: Into<(String, PrismaValue)>,
    Cursor: Into<(String, PrismaValue)>,
{
    pub fn new(where_params: Vec<Where>) -> Self {
        Self {
            where_params,
            with_params: vec![],
            order_by_params: vec![],
            cursor_params: vec![],
            skip: None,
            take: None,
        }
    }

    pub fn with(mut self, param: impl Into<With>) -> Self {
        self.with_params.push(param.into());
        self
    }

    pub fn order_by(mut self, param: impl Into<OrderBy>) -> Self {
        self.order_by_params.push(param.into());
        self
    }

    pub fn cursor(mut self, param: impl Into<Cursor>) -> Self {
        self.cursor_params.push(param.into());
        self
    }

    pub fn skip(mut self, skip: i64) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn take(mut self, take: i64) -> Self {
        self.take = Some(take);
        self
    }

    pub fn to_graphql(self) -> (Vec<(String, QueryValue)>, Vec<Selection>) {
        let Self {
            where_params,
            with_params,
            order_by_params,
            cursor_params,
            skip,
            take,
        } = self;

        let (mut arguments, mut nested_selections) = (vec![], vec![]);

        if with_params.len() > 0 {
            nested_selections = with_params.into_iter().map(Into::into).collect()
        }

        if where_params.len() > 0 {
            arguments.push((
                "where".to_string(),
                PrismaValue::Object(
                    where_params
                        .into_iter()
                        .map(Into::<SerializedWhere>::into)
                        .map(Into::into)
                        .collect(),
                )
                .into(),
            ));
        }

        if order_by_params.len() > 0 {
            arguments.push((
                "orderBy".to_string(),
                PrismaValue::Object(order_by_params.into_iter().map(Into::into).collect()).into(),
            ));
        }

        if cursor_params.len() > 0 {
            arguments.push((
                "cursor".to_string(),
                PrismaValue::Object(cursor_params.into_iter().map(Into::into).collect()).into(),
            ));
        }

        skip.map(|skip| arguments.push(("skip".to_string(), QueryValue::Int(skip))));
        take.map(|take| arguments.push(("take".to_string(), QueryValue::Int(take))));

        (arguments, nested_selections)
    }
}
