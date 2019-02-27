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

// Macro that lets us define weechat sync data structures with their name and fields,
// will generate a parse method that turns an Hdata into a Result<$name>
macro_rules! sync_struct {
    ($name: ident {$($field : ident: $type : ty;)*}) => {

        // Define the structure to have all public fields (for ease of use)
        #[derive(Debug)]
        pub struct $name {
            $(
                pub $field: $type,
            )*
        }

        impl $name {
            pub fn parse(data: &message::Hdata, index: usize) -> Result<$name, SyncError> {
                // Get all fields out of the hdata with their correct types and make sure they worked
                $(
                    let $field = data.get::<$type>(index, stringify!($field));
                    if $field.is_none() {
                        return Err(SyncError{
                            error: SyncErrorType::InvalidData,
                            message: stringify!($field, " expected Some").to_owned(),
                            trace: Backtrace::new()
                        });
                    }
                )*

                // Then just send off the new object!
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
    number: i32;
    full_name: WeechatString;
    short_name: WeechatString;
    nicklist: i32;
    title: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferMoved {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferMerged {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferUnmerged {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferHidden {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferUnhidden {
    number: i32;
    full_name: WeechatString;
    prev_buffer: u128;
    next_buffer: u128;
});

sync_struct!(BufferRenamed {
    number: i32;
    full_name: WeechatString;
    short_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferTitleChanged {
    number: i32;
    full_name: WeechatString;
    title: WeechatString;
});

sync_struct!(BufferCleared {
    number: i32;
    full_name: WeechatString;
});

sync_struct!(BufferTypeChanged {
    number: i32;
    full_name: WeechatString;
    r#type: i32;
});

sync_struct!(BufferLocalvarAdded {
    number: i32;
    full_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferLocalvarChanged {
    number: i32;
    full_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferLocalvarRemoved {
    number: i32;
    full_name: WeechatString;
    local_variables: BTreeMap<WeechatString, WeechatString>;
});

sync_struct!(BufferLineAdded {
    buffer: u128;
    date: u128;
    date_printed: u128;
    displayed: bool;
    highlight: bool;
    tags_array: Vec<WeechatString>;
    prefix: WeechatString;
    message: WeechatString;
});

sync_struct!(BufferClosing {
    number: i32;
    full_name: WeechatString;
});

sync_struct!(Nicklist {
    group: bool;
    visible: bool;
    level: i32;
    name: WeechatString;
    color: WeechatString;
    prefix: WeechatString;
    prefix_color: WeechatString;
});

sync_struct!(NicklistDiff {
    _diff: i8;
    group: bool;
    visible: bool;
    level: i32;
    name: WeechatString;
    color: WeechatString;
    prefix: WeechatString;
    prefix_color: WeechatString;
});
