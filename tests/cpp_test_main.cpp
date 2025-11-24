#include "output_cpp/example.hpp"

#include <cassert>
#include <stdio.h>

int main() {
  // Determine host endianness at runtime for informational purposes
  // (ONX_HOST_ORDER is the compile-time equivalent)
  uint16_t word = 0x00FF;
  uint8_t *byte_ptr = (uint8_t *)&word;

  assert(*byte_ptr == 0xFF);
  assert(ONYX_NETWORK_ORDER == ONYX_BIG_ENDIAN);
  assert(ONYX_HOST_ORDER == ONYX_LITTLE_ENDIAN);

  // --- CALCULATED WIRE BUFFER (BIG ENDIAN FORMAT) ---
  // This 17-byte buffer is constructed to match your asserted target values.
  // If the deserializer correctly swaps bytes from Big to Little Endian, the
  // assertions will pass.

  // 1. id (u64, Target: 0x0807060504030201) -> Wire: [8, 7, 6, 5, 4, 3, 2, 1]
  // 2. name (u8:7=4), yes (bool:1=true) -> Wire: [9] (0b00001001)
  // 3. email (u32, Target: 0x0D0C0B0A) -> Wire: [13, 12, 11, 10]
  // 4. hdr (Header: version=15, checksum=3856, tag=10) -> Wire: [240, 207, 16,
  // 10]

  onyx::User::Buffer buf = {
      8,   7,  6,  5,  4, 3, 2, 1, // id (u64)
      137,                         // name/yes (u8 bit-field)
      13,  12, 11, 10,             // email (u32)
      14,  16, 15, 1               // hdr (Header, 4 packed bytes)
  };

  // Deserialize the buffer
  onyx::User *user = onyx::User::Deserialize(buf);

  assert(user->id() == 578437695752307201ULL);
  assert(user->name() == 9);
  assert(user->yes() == true);
  assert(user->email() == 218893066);
  assert(user->hdr().version() == 14);
  assert(user->hdr().checksum() == 4111);
  assert(user->hdr().tag() == onyx::Status::Active);

  user->id(1681321687ULL);
  user->name(2);
  user->yes(false);
  user->email(34764);
  user->hdr().version(0);
  user->hdr().checksum(300);
  user->hdr().tag(onyx::Status::Error);

  assert(user->id() == 1681321687ULL);
  assert(user->name() == 2);
  assert(user->yes() == false);
  assert(user->email() == 34764);
  assert(user->hdr().version() == 0);
  assert(user->hdr().checksum() == 300);
  assert(user->hdr().tag() == onyx::Status::Error);

  return 0;
}
