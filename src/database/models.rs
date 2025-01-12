use diesel::prelude::*;

use super::schema::{attachment, macro_};

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = macro_)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Macro {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub channel_id: String,
    pub message_id: String,
    pub content: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Macro))]
#[diesel(table_name = attachment)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Attachment {
    pub id: i32,
    pub macro_id: i32,
    pub link: String,
}

#[derive(Insertable)]
#[diesel(table_name = macro_)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMacro<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub content: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = attachment)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewAttachment<'a> {
    pub macro_id: i32,
    pub link: &'a str,
}
