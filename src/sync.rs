use backtrace::Backtrace;
use crate::message;
use message;
use message::WeechatString;

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

#[derive(Debug)]
pub struct BufferLineAdded {
    pub buffer: u128,
    pub date: u128,
    pub date_printed: u128,
    pub displayed: bool,
    pub highlight: bool,
    pub tags_array: Vec<WeechatString>,
    pub prefix: WeechatString,
    pub message: WeechatString,
}

#[derive(Debug)]
pub struct Nicklist {
    pub group: bool,
    pub visible: bool,
    pub level: i32,
    pub name: WeechatString,
    pub color: WeechatString,
    pub prefix: WeechatString,
    pub prefix_color: WeechatString,
}

macro_rules! assert_some_all {
    ($($var : ident), *) => {
        $(
            if $var.is_none() {
                return Err(SyncError{
                    error: SyncErrorType::InvalidData,
                    message: stringify!($var, " expected Some").to_owned(),
                    trace: Backtrace::new()
                });
            }
        )*
    };
}

impl BufferLineAdded {
    pub fn parse(data: &message::Hdata, index: usize) -> Result<BufferLineAdded, SyncError> {
        let buffer = data.get::<u128>(index, "buffer");
        let date = data.get::<u128>(index, "date");
        let date_printed = data.get::<u128>(index, "date_printed");
        let displayed = data.get::<bool>(index, "displayed");
        let highlight = data.get::<bool>(index, "highlight");
        let prefix = data.get::<WeechatString>(index, "prefix");
        let message = data.get::<WeechatString>(index, "message");
        let tags_array = data.get::<Vec<WeechatString>>(index, "tags_array");

        // Make sure everything exists
        assert_some_all!(
            buffer,
            date,
            date_printed,
            displayed,
            highlight,
            tags_array,
            prefix,
            message
        );

        Ok(BufferLineAdded {
            buffer: buffer.unwrap(),
            date: date.unwrap(),
            date_printed: date_printed.unwrap(),
            displayed: displayed.unwrap(),
            highlight: highlight.unwrap(),
            tags_array: tags_array.unwrap(),
            prefix: prefix.unwrap(),
            message: message.unwrap(),
        })
    }
}

impl Nicklist {
    pub fn parse(data: &message::Hdata, index: usize) -> Result<Nicklist, SyncError> {
        let group = data.get::<bool>(index, "group");
        let visible = data.get::<bool>(index, "visible");
        let level = data.get::<i32>(index, "level");
        let name = data.get::<WeechatString>(index, "name");
        let color = data.get::<WeechatString>(index, "color");
        let prefix = data.get::<WeechatString>(index, "prefix");
        let prefix_color = data.get::<WeechatString>(index, "prefix_color");

        // Make sure everything exists
        assert_some_all!(group, visible, level, name, color, prefix, prefix_color);

        Ok(Nicklist {
            group: group.unwrap(),
            visible: visible.unwrap(),
            level: level.unwrap(),
            name: name.unwrap(),
            color: color.unwrap(),
            prefix: prefix.unwrap(),
            prefix_color: prefix_color.unwrap(),
        })
    }
}
