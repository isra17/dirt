#include <string>

extern "C" {

#define CPP_CANDIDATE(CLASS, NAME, DESC, ...) \
  extern const char test_##NAME##_expect[] = #CLASS "::" #DESC; \
  extern CLASS* test_##NAME##_fn

CPP_CANDIDATE(std::string, std_string_string, string())
(std::string *self) { return new (self)std::string(); }

CPP_CANDIDATE(std::string, std_string_string_copy, string(str))
(std::string *self, std::string* s) { return new (self)std::string(*s); }

CPP_CANDIDATE(std::string, std_string_string_substr, string(str,pos,len))
(std::string *self, std::string* s, size_t p, size_t l) {
  return new (self)std::string(*s, p, l);
}

CPP_CANDIDATE(std::string, std_string_string_cstring, string(s))
(std::string *self, char* s) { return new (self)std::string(s); }

CPP_CANDIDATE(std::string, std_string_string_seq, string(s,n))
(std::string *self, char* s, size_t n) { return new (self)std::string(s, n); }

CPP_CANDIDATE(std::string, std_string_string_fill, string(n,c))
(std::string *self, size_t n, char c) { return new (self)std::string(n, c); }

CPP_CANDIDATE(std::string, std_string_string_append_cstring, append(s))
(std::string *self, char* s) { return new self->append(s); }

}

int main() {
  return 0;
}
