#include <string>

extern "C" {

extern const char test_std_string_string_expect[] = "std::string::string()";
extern std::string* test_std_string_string_fn(std::string *self) {
  return new (self)std::string();
}

extern const char test_std_string_string_char_expect[] = "std::string::string(char*)";
extern std::string* test_std_string_string_char_fn(std::string* self, char *s) {
  return new (self)std::string(s);
}

}

int main() {
  return 0;
}
