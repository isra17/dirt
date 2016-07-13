#include <string>

extern "C" {

extern const char test_std_string_string_expect[] = "std::string::string()";
extern std::string* test_std_string_string_fn(std::string *s) {
  return new (s)std::string();
}

}

int main() {
  return 0;
}
