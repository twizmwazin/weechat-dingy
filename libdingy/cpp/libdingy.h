#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  None,
  Zlib,
} CompressionType;

typedef enum {
  Buffers,
  Upgrade,
  Buffer,
  Nicklist,
} SyncOption;

typedef enum {
  CharType,
  IntType,
  LongType,
  StringType,
  BufferType,
  PointerType,
  TimeType,
  HashTableType,
  HdataType,
  InfoType,
  InfoListType,
  ArrayType,
} WeechatTypeEnum;

typedef struct Hdata Hdata;

typedef struct Message Message;

typedef struct WeechatType WeechatType;

/**
 * Create a desync command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param argument_buffers: List of strings or null
 * @param argument_options: List of options or null
 * @param argument_buffers_lengths: List of lengths of arguments or null
 * @param arguments_length: Number of arguments
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_desync_print(const uint8_t *id,
                               uintptr_t id_length,
                               const uint8_t *const *argument_buffers,
                               const SyncOption *argument_options,
                               const uintptr_t *argument_buffers_lengths,
                               uintptr_t arguments_length,
                               uint8_t *output,
                               uintptr_t output_length);

/**
 * Create an hdata command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param hdata: Name of hdata
 * @param hdata_length: Length of hdata string
 * @param pointer: Pointer to infolist or null
 * @param pointer_length: Length of pointer string
 * @param pointer_count: Count for pointer parameter
 * @param var_names: List of strings or null
 * @param var_counts: List of counts or null
 * @param var_names_lengths: List of lengths of vars or null
 * @param vars_length: Number of vars
 * @param keys: List of strings or null
 * @param keys_lengths: List of lengths of keys or null
 * @param keys_length: Number of keys
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_hdata_print(const uint8_t *id,
                              uintptr_t id_length,
                              const uint8_t *hdata,
                              uintptr_t hdata_length,
                              const uint8_t *pointer,
                              uintptr_t pointer_length,
                              const int32_t *pointer_count,
                              const uint8_t *const *var_names,
                              const int32_t *const *var_counts,
                              const uintptr_t *var_names_lengths,
                              uintptr_t vars_length,
                              const uint8_t *const *keys,
                              const uintptr_t *keys_lengths,
                              uintptr_t keys_length,
                              uint8_t *output,
                              uintptr_t output_length);

/**
 * Create an info command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param name: Info name
 * @param name_length: Length of name string
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_info_print(const uint8_t *id,
                             uintptr_t id_length,
                             const uint8_t *name,
                             uintptr_t name_length,
                             uint8_t *output,
                             uintptr_t output_length);

/**
 * Create an infolist command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param name: Name of infolist
 * @param name_length: Length of name string
 * @param pointer: Pointer to infolist or null
 * @param pointer_length: Length of pointer string
 * @param arguments: List of strings or null
 * @param arguments_lengths: List of lengths of arguments or null
 * @param arguments_length: Number of arguments
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_infolist_print(const uint8_t *id,
                                 uintptr_t id_length,
                                 const uint8_t *name,
                                 uintptr_t name_length,
                                 const uint8_t *pointer,
                                 uintptr_t pointer_length,
                                 const uint8_t *const *arguments,
                                 const uintptr_t *arguments_lengths,
                                 uintptr_t arguments_length,
                                 uint8_t *output,
                                 uintptr_t output_length);

/**
 * Create an init command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param password: Password for server or null
 * @param password_length: Length of password string
 * @param compression: Compression type or null
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_init_print(const uint8_t *id,
                             uintptr_t id_length,
                             const uint8_t *password,
                             uintptr_t password_length,
                             const CompressionType *compression,
                             uint8_t *output,
                             uintptr_t output_length);

/**
 * Create an input command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param buffer: Buffer for input or null
 * @param buffer_length: Length of buffer string
 * @param data: Buffer for data or null
 * @param data_length: Length of data string
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_input_print(const uint8_t *id,
                              uintptr_t id_length,
                              const uint8_t *buffer,
                              uintptr_t buffer_length,
                              const uint8_t *data,
                              uintptr_t data_length,
                              uint8_t *output,
                              uintptr_t output_length);

/**
 * Create a nicklist command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param buffer: Buffer for list or null
 * @param buffer_length: Length of buffer string
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_nicklist_print(const uint8_t *id,
                                 uintptr_t id_length,
                                 const uint8_t *buffer,
                                 uintptr_t buffer_length,
                                 uint8_t *output,
                                 uintptr_t output_length);

/**
 * Create a ping command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param arguments: List of strings or null
 * @param arguments_lengths: List of lengths of arguments or null
 * @param arguments_length: Number of arguments
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_ping_print(const uint8_t *id,
                             uintptr_t id_length,
                             const uint8_t *const *arguments,
                             const uintptr_t *arguments_lengths,
                             uintptr_t arguments_length,
                             uint8_t *output,
                             uintptr_t output_length);

/**
 * Create a quit command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_quit_print(const uint8_t *id,
                             uintptr_t id_length,
                             uint8_t *output,
                             uintptr_t output_length);

/**
 * Create a sync command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param argument_buffers: List of strings or null
 * @param argument_options: List of options or null
 * @param argument_buffers_lengths: List of lengths of arguments or null
 * @param arguments_length: Number of arguments
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_sync_print(const uint8_t *id,
                             uintptr_t id_length,
                             const uint8_t *const *argument_buffers,
                             const SyncOption *argument_options,
                             const uintptr_t *argument_buffers_lengths,
                             uintptr_t arguments_length,
                             uint8_t *output,
                             uintptr_t output_length);

/**
 * Create a test command
 * @param id: Id of command or null
 * @param id_length: Length of id string
 * @param output: Output buffer
 * @param output_length: Capacity of output buffer
 * @return Number of bytes in full message (even if truncated)
 */
uintptr_t command_test_print(const uint8_t *id,
                             uintptr_t id_length,
                             uint8_t *output,
                             uintptr_t output_length);

/**
 * Get count of buffers in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @return Number of buffers in hdata
 */
uintptr_t hdata_buffer_count(Hdata *hdata);

/**
 * Get buffer object in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @param buffer_index: Index of buffer
 * @param key_index: Index of key
 * @return Pointer to object item
 */
WeechatType *hdata_buffer_object_item(Hdata *hdata, uintptr_t buffer_index, uintptr_t key_index);

/**
 * Get buffer path item in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @param buffer_index: Index of buffer
 * @param path_index: Index of path item
 * @return Pointer to path item
 */
WeechatType *hdata_buffer_path_item(Hdata *hdata, uintptr_t buffer_index, uintptr_t path_index);

/**
 * Get count of keys in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @return Number of keys in hdata
 */
uintptr_t hdata_keys_count(Hdata *hdata);

/**
 * Get name of key in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @param index: Index of key
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *hdata_keys_item(Hdata *hdata, uintptr_t index, uintptr_t *length);

/**
 * Get count of h_path in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @return Number of items in hdata's h_path
 */
uintptr_t hdata_path_count(Hdata *hdata);

/**
 * Get item of h_path in an Hdata
 * @param hdata: Pointer to Hdata struct
 * @param index: Index of item in path
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *hdata_path_item(Hdata *hdata, uintptr_t index, uintptr_t *length);

/**
 * Get number of data items in message
 * @param message: Message
 * @return Number of data items
 */
uintptr_t message_data_count(Message *message);

/**
 * Get message's data item at index
 * @param message: Message
 * @param index: Index of data item
 * @return Pointer to data item, or null
 */
WeechatType *message_data_item(Message *message, uintptr_t index);

/**
 * Free a message and all associated data structures
 * @param message: Message to free
 */
void message_free(Message *message);

/**
 * Get id of message
 * @param message: Message
 * @param length: Length of id buffer
 * @return Buffer with id (not null-terminated)
 */
const uint8_t *message_id(const Message *message, uintptr_t *length);

/**
 * Parse a message from bytes
 * @param bytes: List of bytes for message
 * @param length: Length of bytes
 * @param parse_length: Length of parsed data
 * @return Message structure pointer, or null
 */
Message *message_parse(const uint8_t *bytes, uintptr_t length, uintptr_t *parse_length);

/**
 * Parse a message header
 * @param bytes: List of bytes for message
 * @param length: Length of bytes
 * @return Full length of expected message data
 */
uintptr_t message_parse_length(const uint8_t *bytes, uintptr_t length);

/**
 * Get number of items in a WeechatType::Array
 * @param weechat_type: WeechatType pointer
 * @return Number of array items
 */
uintptr_t weechat_type_array_count(WeechatType *weechat_type);

/**
 * Get item in a WeechatType::Array
 * @param weechat_type: WeechatType pointer
 * @paarm index: Index of item in array
 * @return Pointer to array item
 */
WeechatType *weechat_type_array_item(WeechatType *weechat_type, uintptr_t index);

/**
 * Get buffer value from a WeechatType::Buffer
 * @param weechat_type: WeechatType pointer
 * @param length: Pointer to length of buffer
 * @return Buffer (not null-terminated)
 */
const uint8_t *weechat_type_buffer_get(WeechatType *weechat_type, uintptr_t *length);

/**
 * Get char value from a WeechatType::Char
 * @param weechat_type: WeechatType pointer
 * @return Char value
 */
int8_t weechat_type_char_get(WeechatType *weechat_type);

/**
 * Get enum type of WeechatType pointer
 * @param weechat_type: WeechatType pointer
 * @return Which enum type it is
 */
WeechatTypeEnum weechat_type_enum(const WeechatType *weechat_type);

/**
 * Get count of WeechatType::HashTable
 * @param weechat_type: WeechatType pointer
 * @return Number of entries in hash table
 */
uintptr_t weechat_type_hash_table_count(WeechatType *weechat_type);

/**
 * Get entry key from a WeechatType::HashTable
 * @param weechat_type: WeechatType pointer
 * @param index: Index of entry
 * @return Entry key pointer
 */
WeechatType *weechat_type_hash_table_get_key(WeechatType *weechat_type, uintptr_t index);

/**
 * Get entry value from a WeechatType::HashTable
 * @param weechat_type: WeechatType pointer
 * @param index: Index of entry
 * @return Entry value pointer
 */
WeechatType *weechat_type_hash_table_get_value(WeechatType *weechat_type, uintptr_t index);

/**
 * Get pointer to an Hdata struct from a WeechatType::Hdata
 * @param weechat_type: WeechatType pointer
 * @return Pointer to Hdata struct
 */
Hdata *weechat_type_hdata_get(WeechatType *weechat_type);

/**
 * Get info name from a WeechatType::Info
 * @param weechat_type: WeechatType pointer
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *weechat_type_info_get_name(WeechatType *weechat_type, uintptr_t *length);

/**
 * Get info value from a WeechatType::Info
 * @param weechat_type: WeechatType pointer
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *weechat_type_info_get_value(WeechatType *weechat_type, uintptr_t *length);

/**
 * Get number of items in a WeechatType::InfoList
 * @param weechat_type: WeechatType pointer
 * @return Number of infolist items
 */
uintptr_t weechat_type_info_list_count(WeechatType *weechat_type);

/**
 * Get name of a WeechatType::InfoList
 * @param weechat_type: WeechatType pointer
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *weechat_type_info_list_get_name(WeechatType *weechat_type, uintptr_t *length);

/**
 * Get number of entries in an item in a WeechatType::InfoList
 * @param weechat_type: WeechatType pointer
 * @param index: Index of item
 * @return Number of item entries
 */
uintptr_t weechat_type_info_list_item_count(WeechatType *weechat_type, uintptr_t index);

/**
 * Get entry name in an item in a WeechatType::InfoList
 * @param weechat_type: WeechatType pointer
 * @param item_index: Index of item
 * @param entry_index: Index of entry
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *weechat_type_info_list_item_item_get_name(WeechatType *weechat_type,
                                                         uintptr_t item_index,
                                                         uintptr_t entry_index,
                                                         uintptr_t *length);

/**
 * Get entry value in an item in a WeechatType::InfoList
 * @param weechat_type: WeechatType pointer
 * @param item_index: Index of item
 * @param entry_index: Index of entry
 * @return Pointer to entry value
 */
WeechatType *weechat_type_info_list_item_item_get_value(WeechatType *weechat_type,
                                                        uintptr_t item_index,
                                                        uintptr_t entry_index);

/**
 * Get int value from a WeechatType::Int
 * @param weechat_type: WeechatType pointer
 * @return Int value
 */
int32_t weechat_type_int_get(WeechatType *weechat_type);

/**
 * Get long value from a WeechatType::Long
 * @param weechat_type: WeechatType pointer
 * @return Long value
 */
intptr_t weechat_type_long_get(WeechatType *weechat_type);

/**
 * Get pointer value from a WeechatType::Pointer
 * @param weechat_type: WeechatType pointer
 * @return Pointer value
 */
uintptr_t weechat_type_pointer_get(WeechatType *weechat_type);

/**
 * Get string value from a WeechatType::String
 * @param weechat_type: WeechatType pointer
 * @param length: Pointer to length of string
 * @return String buffer (not null-terminated)
 */
const uint8_t *weechat_type_string_get(WeechatType *weechat_type, uintptr_t *length);

/**
 * Get time value from a WeechatType::Time
 * @param weechat_type: WeechatType pointer
 * @return Time value
 */
uintptr_t weechat_type_time_get(WeechatType *weechat_type);
