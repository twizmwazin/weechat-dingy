#include <stdio.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <stdlib.h>
#include <stdbool.h>
#include <errno.h>
#include <string.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <string>
#include <vector>

extern "C" {
#include "libdingy.h"
}

void print_strbuf(const uint8_t *str, const uintptr_t length) {
	if (str == nullptr) {
		printf("nullptr");
	} else {
		for (int j = 0; j < length; j++) {
			printf("%c", str[j]);
		}
	}
}

void print_buffer(const uint8_t *buffer, const uintptr_t length) {
	if (buffer == nullptr) {
		printf("nullptr");
	} else {
		for (int j = 0; j < length; j++) {
			if (j > 0) {
				printf(", ");
			}
			printf("0x%02hhx", buffer[j]);
		}
	}
}

template<typename F, typename ... Args>
void print_string(F fn, Args... args) {
	uintptr_t str_length;
	const uint8_t *str = fn(args..., &str_length);
	print_strbuf(str, str_length);
}

void print_type(WeechatType *type) {
	WeechatTypeEnum which = weechat_type_enum(type);
	switch (which) {
		case CharType:
			printf("Char(%c)", weechat_type_char_get(type));
			break;
		case IntType:
			printf("Int(%d)", weechat_type_int_get(type));
			break;
		case LongType:
			printf("Long(%zu)", weechat_type_long_get(type));
			break;
		case StringType: {
			printf("String(");
			print_string(weechat_type_string_get, type);
			printf(")");
			break;
		}
		case BufferType: {
			printf("Buffer(");
			print_string(weechat_type_buffer_get, type);
			printf(")");
			break;
		}
		case PointerType:
			printf("Pointer(%p)", (void *)weechat_type_pointer_get(type));
			break;
		case TimeType:
			printf("Time(%zu)", weechat_type_time_get(type));
			break;
		case HashTableType: {
			printf("HashTable(");

			uintptr_t count = weechat_type_hash_table_count(type);
			for (int i = 0; i < count; ++i) {
				if (i > 0) {
					printf(", ");
				}
				WeechatType *key = weechat_type_hash_table_get_key(type, i);
				print_type(key);

				printf(":");

				WeechatType *value = weechat_type_hash_table_get_value(type, i);
				print_type(value);
			}

			printf(")");
			break;
		}
		case HdataType: {
			printf("Hdata(");
			Hdata *hdata = weechat_type_hdata_get(type);

			uintptr_t path_count = hdata_path_count(hdata);
			for (int i = 0; i < path_count; ++i) {
				if (i > 0) {
					printf("/");
				}
				print_string(hdata_path_item, hdata, i);
			}
			printf(",");

			uintptr_t keys_count = hdata_keys_count(hdata);

			uintptr_t buffer_count = hdata_buffer_count(hdata);
			for (int i = 0; i < buffer_count; ++i) {
				if (i > 0) {
					printf(", ");
				}
				for (int j = 0; j < path_count; ++j) {
					if (j > 0) {
						printf("/");
					}
					WeechatType *item = hdata_buffer_path_item(hdata, i, j);
					print_type(item);
				}

				printf(":");
				printf("[");

				for (int j = 0; j < keys_count; ++j) {
					if (j > 0) {
						printf(", ");
					}
					print_string(hdata_keys_item, hdata, j);
					printf(":");
					WeechatType *item = hdata_buffer_object_item(hdata, i, j);
					print_type(item);
				}
				printf("]");
			}

			printf(")");
			break;
		}
		case InfoType: {
			printf("Info(");
			print_string(weechat_type_info_get_name, type);
			printf(", ");
			print_string(weechat_type_info_get_value, type);
			printf(")");
			break;
		}
		case InfoListType: {
			printf("Infolist(");
			print_string(weechat_type_info_list_get_name, type);
			printf(", ");

			uintptr_t count = weechat_type_info_list_count(type);
			for (int i = 0; i < count; ++i) {
				if (i > 0) {
					printf(", ");
				}
				printf("[");

				uintptr_t item_count = weechat_type_info_list_item_count(type, i);
				for (int j = 0; j < item_count; ++j) {
					if (j > 0) {
						printf(", ");
					}
					print_string(weechat_type_info_list_item_item_get_name, type, i, j);
					printf(":");

					WeechatType *value = weechat_type_info_list_item_item_get_value(type, i, j);
					print_type(value);
				}
				printf("]");
			}

			printf(")");
			break;
		}
		case ArrayType: {
			printf("Array(");

			uintptr_t count = weechat_type_array_count(type);
			for (int i = 0; i < count; ++i) {
				if (i > 0) {
					printf(", ");
				}
				WeechatType *item = weechat_type_array_item(type, i);
				print_type(item);
			}

			printf(")");
			break;
		}
	}
}

void print_message(Message *msg) {
	print_string(message_id, msg);
	printf(": ");

	uintptr_t data_count = message_data_count(msg);
	printf("%zu data:\n", data_count);

	for (int i = 0; i < data_count; ++i) {
		WeechatType *item = message_data_item(msg, i);
		print_type(item);
		printf("\n");
	}
}

int main(int argc, const char **argv) {
	uint8_t bytes[] = {0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x07, 0x42, 0x75, 0x66, 0x66, 0x65, 0x72, 0x73, 0x68, 0x64, 0x61, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF1};

	uintptr_t parsed;
	message_parse(bytes, sizeof(bytes), &parsed);

	char *server = getenv("server");
	char *password = getenv("password");

	char address[128];
	short tcp_port;

	sscanf(server, "%[^:]:%hd", address, &tcp_port);

	struct sockaddr_in tcp_client;

	int tcp_sock = socket(PF_INET, SOCK_STREAM, 0);
	if (tcp_sock < 0) {
		perror("tcp socket()");
		return EXIT_FAILURE;
	}

	tcp_client.sin_family = PF_INET;
	tcp_client.sin_port = htons(tcp_port);
	inet_pton(AF_INET, address, &tcp_client.sin_addr);
	if (connect(tcp_sock, (struct sockaddr *)&tcp_client, sizeof(tcp_client)) < 0) {
		perror("tcp bind()");
		return EXIT_FAILURE;
	}

	//Don't clog up the port
	int value = 1;
	setsockopt(tcp_sock, SOL_SOCKET, SO_REUSEADDR, (const void *)&value , sizeof(int));

	uint8_t init[2048];
	CompressionType compression = CompressionType::Zlib;
	uintptr_t init_length = command_init_print((const uint8_t *)"aaa", 3, (const uint8_t *)"jack2istheworst", 15, &compression, init, 2047);
	init[init_length] = 0;
	printf("%s", init);
	send(tcp_sock, init, strlen((char *)init), 0);

	uint8_t info[2048];
	uintptr_t info_length = command_info_print((const uint8_t *)"bbb", 3, (const uint8_t *)"version", 7, info, 2047);
	info[info_length] = 0;
	printf("%s", info);
	send(tcp_sock, info, strlen((char *)info), 0);

	uint8_t ping[2048];
	uint8_t *ping_args[] = {(uint8_t *)"test", (uint8_t *)"test2"};
	uintptr_t ping_args_lengths[] = {4, 5};
	uintptr_t ping_length = command_ping_print((const uint8_t *)"ddd", 3, ping_args, ping_args_lengths, 2, ping, 2047);
	ping[ping_length] = 0;
	printf("%s", ping);
	send(tcp_sock, ping, strlen((char *)ping), 0);

	uint8_t sync[2048];
	uintptr_t sync_length = command_sync_print((const uint8_t *)"eee", 3, nullptr, nullptr, nullptr, 0, sync, 2047);
	sync[sync_length] = 0;
	printf("%s", sync);
	send(tcp_sock, sync, strlen((char *)sync), 0);

//	uint8_t desync[2048];
//	uint8_t *desync_buffers[] = {(uint8_t *)"test1", (uint8_t *)"test2"};
//	SyncOption desync_options[] = {SyncOption::Buffer, SyncOption::Nicklist};
//	uintptr_t desync_buffers_lengths[] = {5, 5};
//	uintptr_t desync_length = command_desync_print((const uint8_t *)"fff", 3, desync_buffers, desync_options, desync_buffers_lengths, 2, desync, 2047);
//	desync[desync_length] = 0;
//	printf("%s", desync);
//	send(tcp_sock, desync, strlen((char *)desync), 0);

	uint8_t nicklist[2048];
	uintptr_t nicklist_length = command_nicklist_print((const uint8_t *)"ggg", 3, nullptr, 0, nicklist, 2047);
	nicklist[nicklist_length] = 0;
	printf("%s", nicklist);
	send(tcp_sock, nicklist, strlen((char *)nicklist), 0);

	uint8_t input[2048];
	uintptr_t input_length = command_input_print((const uint8_t *)"hhh", 3, (const uint8_t *)"irc.rpisec.#dingy", 17, (const uint8_t *)"message", 7, input, 2047);
	input[input_length] = 0;
	printf("%s", input);
	send(tcp_sock, input, strlen((char *)input), 0);

//	uint8_t infolist[2048];
//	uint8_t *infolist_args[] = {(uint8_t *)"test", (uint8_t *)"test2"};
//	uintptr_t infolist_args_lengths[] = {4, 5};
//	uintptr_t infolist_length = command_infolist_print((const uint8_t *)"iii", 3, (const uint8_t *)"buffers", 7, (const uint8_t *)"0x1234", 6, infolist_args, infolist_args_lengths, 2, infolist, 2047);
//	infolist[infolist_length] = 0;
//	printf("%s", infolist);
//	send(tcp_sock, infolist, strlen((char *)infolist), 0);

	uint8_t hdata[2048];
	uint32_t pointer_count = 0;
	uint32_t var1_count = 0;
	uint32_t var2_count = 3;
	uint8_t *hdata_var_names[] = {(uint8_t *)"lines", (uint8_t *)"first_line", (uint8_t *)"data"};
	uint32_t *hdata_var_counts[] = {&var1_count, &var2_count, nullptr};
	uintptr_t hdata_vars_lengths[] = {5, 10, 4};
	uint8_t *hdata_keys[] = {(uint8_t *)"full_name", (uint8_t *)"test2"};
	uintptr_t hdata_keys_lengths[] = {9, 5};
	uintptr_t hdata_length = command_hdata_print((const uint8_t *)"jjj", 3, (const uint8_t *)"buffer", 6, (const uint8_t *)"gui_buffers", 11, &pointer_count, hdata_var_names, hdata_var_counts, hdata_vars_lengths, 3, hdata_keys, hdata_keys_lengths, 2, hdata, 2047);
	hdata[hdata_length] = 0;
	printf("%s", hdata);
	send(tcp_sock, hdata, strlen((char *)hdata), 0);

//	uint8_t quit[2048];
//	uintptr_t quit_length = command_quit_print((const uint8_t *)"ccc", 3, quit, 2047);
//	quit[quit_length] = 0;
//	printf("%s", quit);
//	send(tcp_sock, quit, strlen((char *)quit), 0);

	std::vector<uint8_t> data;

	while (true) {
		uint8_t buffer[1024];
		ssize_t rcvd = recv(tcp_sock, buffer, 1024, 0);
		if (rcvd == 0) {
			//EOF
			break;
		}
		if (rcvd < 0) {
			//Error
			if (errno == EINTR) {
				continue;
			}
			perror("recv");
			break;
		}

		for (int i = 0; i < rcvd; ++i) {
			data.push_back(buffer[i]);
		}

		uintptr_t parse_length;
		do {
			Message *msg = message_parse(data.data(), data.size(), &parse_length);
			if (msg != nullptr) {
				data.erase(data.begin(), data.begin() + parse_length);
				print_message(msg);
				message_free(msg);
			} else {
				break;
			}
		} while (parse_length > 0 && data.size() > 0);
	}

	close(tcp_sock);
}
