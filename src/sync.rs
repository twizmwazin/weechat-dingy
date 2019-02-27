use crate::message;
use backtrace::Backtrace;
use message::WeechatString;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum SyncErrorType {
    InvalidData,
    Other,
}

#[derive(Constructor, Debug)]
pub struct SyncError {
    pub error: SyncErrorType,
    pub message: String,
    pub trace: Backtrace,
}

macro_rules! sync_struct {
    ($name: ident {$($field : ident: $type : ty),*}) => {

        #[derive(Debug)]
        pub struct $name {
            $(
                pub $field: $type,
            )*
        }
        impl $name {
            pub fn parse(data: &message::Hdata, index: usize) -> Result<$name, SyncError> {
                $(
                    let $field = data.get::<$type>(index, stringify!($field));
                )*

                $(
                    if $field.is_none() {
                        return Err(SyncError{
                            error: SyncErrorType::InvalidData,
                            message: stringify!($field, " expected Some").to_owned(),
                            trace: Backtrace::new()
                        });
                    }
                )*

                Ok($name{
                    $(
                        $field: $field.unwrap(),
                    )*
                })
            }
        }
    };
}

sync_struct!(BufferOpened {
    number: i32,
    full_name: WeechatString,
    short_name: WeechatString,
    nicklist: i32,
    title: WeechatString,
    local_variables: BTreeMap<WeechatString, WeechatString>,
    prev_buffer: u128,
    next_buffer: u128
});

sync_struct!(BufferMoved {
    number: i32,
    full_name: WeechatString,
    prev_buffer: u128,
    next_buffer: u128
});

sync_struct!(BufferLineAdded {
    buffer: u128,
    date: u128,
    date_printed: u128,
    displayed: bool,
    highlight: bool,
    prefix: WeechatString,
    message: WeechatString,
    tags_array: Vec<WeechatString>
});

sync_struct!(Nicklist {
    group: bool,
    visible: bool,
    level: i32,
    name: WeechatString,
    color: WeechatString,
    prefix: WeechatString,
    prefix_color: WeechatString
});
