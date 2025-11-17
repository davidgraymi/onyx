#include "data/example.hpp"

#include <cassert>
#include <stdio.h>

int main() {
    // Determine host endianness at runtime for informational purposes (ONX_HOST_ORDER is the compile-time equivalent)
    uint16_t word = 0x00FF;
    uint8_t *byte_ptr = (uint8_t *) &word;
    
    printf("Host System Endianness: %s\n", (*byte_ptr == 0xFF) ? "Little-Endian" : "Big-Endian");
    printf("Network Endianness (from Onyx IDL): %d (Big-Endian is 4321)\n", ONYX_NETWORK_ORDER);
    printf("Host Endianness (from Onyx IDL): %d (Big-Endian is 4321)\n", ONYX_HOST_ORDER);
    
    // --- CALCULATED WIRE BUFFER (BIG ENDIAN FORMAT) ---
    // This 17-byte buffer is constructed to match your asserted target values.
    // If the deserializer correctly swaps bytes from Big to Little Endian, the assertions will pass.
    
    // 1. id (u64, Target: 0x0807060504030201) -> Wire: [8, 7, 6, 5, 4, 3, 2, 1]
    // 2. name (u8:7=4), yes (bool:1=true) -> Wire: [9] (0b00001001)
    // 3. email (u32, Target: 0x0D0C0B0A) -> Wire: [13, 12, 11, 10]
    // 4. hdr (Header: version=15, checksum=3856, tag=10) -> Wire: [240, 207, 16, 10]
    
    onyx::User::Buffer buf = {
        8, 7, 6, 5, 4, 3, 2, 1, // id (u64)
        132,                    // name/yes (u8 bit-field)
        13, 12, 11, 10,         // email (u32)
        14, 16, 15, 1           // hdr (Header, 4 packed bytes)
    };

    printf("\nWire Buffer (size %zu bytes): [", onyx::User::kSizeOf);
    for (size_t i = 0; i < onyx::User::kSizeOf; i++) {
        printf("%u%s", buf[i], (i == onyx::User::kSizeOf - 1) ? "" : ", ");
    }
    printf("]\n\n");

    // Deserialize the buffer
    onyx::User* user = onyx::User::Deserialize(buf);

    printf("Deserialized User Data (Endianness-Corrected) {%lu}:\n", sizeof(onyx::User));
    printf("  id: %llu\n", (unsigned long long)user->id());
    printf("  name: %d\n", user->name());
    printf("  yes: %d\n", user->yes());
    printf("  email: %u\n", user->email());
    printf("  hdr.version: %u\n", user->hdr().version());
    printf("  hdr.checksum: %u\n", user->hdr().checksum());
    printf("  hdr.tag: %hhu\n", (uint8_t)user->hdr().tag());

    printf("\nIn Memory Buffer (size %zu bytes): [", onyx::User::kSizeOf);
    for (size_t i = 0; i < onyx::User::kSizeOf; i++) {
        printf("%u%s", buf[i], (i == onyx::User::kSizeOf - 1) ? "" : ", ");
    }
    printf("]\n\n");

    assert(user->id() == 578437695752307201ULL);
    assert(user->name() == 4);
    assert(user->yes() == true);
    assert(user->email() == 218893066);
    assert(user->hdr().version() == 14);
    assert(user->hdr().checksum() == 4111);
    assert(user->hdr().tag() == onyx::Status::Active);
    assert(user->hdr().tag() == onyx::Status::Inactive);
    
    printf("Deserialization assertions passed!\n\n");

    user->id(1681321687ULL);
    user->name(2);
    user->yes(false);
    user->email(34764);
    user->hdr().version(0);
    user->hdr().checksum(300);
    user->hdr().tag(onyx::Status::Error);

    printf("User Data After Change {%lu}:\n", sizeof(onyx::User));
    printf("  id: %llu\n", (unsigned long long)user->id());
    printf("  name: %d\n", user->name());
    printf("  yes: %d\n", user->yes());
    printf("  email: %u\n", user->email());
    printf("  hdr.version: %u\n", user->hdr().version());
    printf("  hdr.checksum: %u\n", user->hdr().checksum());
    printf("  hdr.tag: %hhu\n", (uint8_t)user->hdr().tag());

    printf("\nIn Memory Buffer (size %zu bytes): [", onyx::User::kSizeOf);
    for (size_t i = 0; i < onyx::User::kSizeOf; i++) {
        printf("%u%s", buf[i], (i == onyx::User::kSizeOf - 1) ? "" : ", ");
    }
    printf("]\n\n");

    assert(user->id() == 1681321687ULL);
    assert(user->name() == 2);
    assert(user->yes() == false);
    assert(user->email() == 34764);
    assert(user->hdr().version() == 0);
    assert(user->hdr().checksum() == 300);
    assert(user->hdr().tag() == onyx::Status::Error);

    printf("\nAll endianness and bit-field assertions passed!\n\n");

    return 0;
}
