# Lit
Lit Programming Language Repository

## Lit is currently in development:
### What can you do in Lit currently:

- Print to console by `print(str s, end='\n')`:

      fun main() {
          print('Hello, World', end='!\n')
      }
- Declare variables with types: `int, float, bool, str`:

      fun main() {
          int a = 10
          float b = 6.28
          bool ok = true
          str greet = 'Hello!'

          print(a)
          print(b)
          print(ok)
          print(greet)
      }
- Arithmetics with int and float type variables:

      fun main() {
          int a = 2 + 2 * 2
          float b = a + 2.5

          print('Can you solve this: 2 + 2 * 2?')
          print('Can you add 2.5 to that?')
      }
- Interpolating strings:

      fun main() {
          int quantity = 10
          str s = 'We have {quantity} apples'
          print(s)
      }
- Comments by `//` and MultiLine Comments with `/* */`:

      fun main() {
          // Comment
          // Also Comment
          // One more

          /* MultiLine
              Comment */
      }
- Incrementing numbers `+=, -=, *=, /=, %=, ++, --`:

      fun main() {
          int a = 10
          a += 5
          a -= 1
          a *= 2
          a /= 2
          a %= 3 // %= is unaccessible for float
          a++
          a--
          
          print(a) // 2
      }