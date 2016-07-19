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

extern const char test_std_string_string_string_expect[] = "std::string::string(std::string*)";
extern std::string* test_std_string_string_string_fn(std::string* self, std::string *s) {
  return new (self)std::string(*s);
}

extern const char test_std_string_char_char_expect[] = "std::string::append(char*)";
extern std::string* test_std_string_char_char_fn(std::string* self, char *s) {
  return &self->append(s);
}

}

int main() {
  return 0;
}
