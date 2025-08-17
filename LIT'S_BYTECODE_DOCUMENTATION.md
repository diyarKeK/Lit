# Документация Байт-кода Lit

## Чтобы начать, создайте файл с расширением `.lbc`

## Пустая программа:
Все программы должны иметь `LABEL main` и `HALT <exit code>`
где точка входа в программу - это `LABEL main`, 
а выход из программы осуществляется с помощью `HALT <exit code>`

    LABEL main

        ; Your Program
    
        HALT 0

Команда `LABEL` это объявления метки
Команда `HALT 0` это выход из программы кодом 0 `Успех`

## Вывод в консоль
    
    LABEL main
        
        ; Добавляем литерал в стек
        PUSH_CONST str "Hello, World!"

        ; выводим значение из стека
        PRINT

        HALT 0

`PUSH_CONST <type> <value>` - складывает в стек типизированное значение.<br>
`<type>` может быть:
- `int` - натуральное число
- `float` - число с плавающей точкой
- `bool` - булевое значение
- `str` - строка

`value` может быть только литералом

`PRINT` - выводит в консоль последнее значение из стека.
Как работает: Добавив в стек литерал с помощью `PUSH_CONST`, 
`PRINT` достает последнее значение из стека с помощью: `stack.pop()`

## Объявление и использование переменных

    LABEL main
        
        PUSH_CONST int 10
        ; объявляем переменную `a` с полседним значением из стека
        STORE_VAR a
    
        PUSH_CONST float 8.5
        STORE_VAR b
        
        PUSH_CONST bool true
        STORE_VAR ok
        
        PUSH_CONST str "Hello!"
        STORE_VAR greet


        ; загружаем переменную в стек
        LOAD_VAR a
        PRINT
        
        LOAD_VAR b
        PRINT

        LOAD_VAR ok
        PRINT

        LOAD_VAR greet
        PRINT

        HALT 0

Чтобы объявить переменную, сначала добавим в стек литерал, потом используем `STORE_VAR <name>`
Так `STORE_VAR` достает последнее значение из стека и объявляет переменную с этим значением

Чтобы использовать переменную, используйте `LOAD_VAR <name>`

## Операции с int, float, str:
Операции:
- `ADD_VAR` = `+=`, для `int, float, str`
- `SUB_VAR` = `-=`
- `MUL_VAR` = `*=`
- `DIV_VAR` = `/=`
- `MOD_VAR` = `%=`, только для `int`
- `INC` = `++`
- `DEC` = `--`

Все они используют значения из стека

    LABEL main
        
        PUSH_CONST int 10
        STORE_VAR a

        PUSH_CONST int 2
        ; добавляем к `a` 2 -> a += 2
        ADD_VAR a

        ; a++
        INC a

        LOAD_VAR a
        PRINT

        HALT 0

## Арифметика
Давайте объявим переменную `a` со значением 15:
    
    PUSH_CONST int 15
    STORE_VAR a

Теперь объявим переменную `b` со значением `a + 12`
Для этого, нам нужно сложить `a` с 12 и положить их в `b`.
Помните как мы сначала вводили значение в стек потом объявляли переменную?
Здесь точно также:

    LABEL main

        PUSH_CONST int 15               ; ввод в стэк
        STORE_VAR a                     ; сохраняем переменную

        LOAD_VAR a                      ; загружаем в стек
        PUSH_CONST int 2                ; добавляем в стек еще и 2
        ADD                             ; складываем 2 последних значения из стека и создаем новый литерал в стеке, убрав с помощью pop() другие значения

        STORE_VAR b                     ; объявляем переменную


Операции:
- `ADD` = `+`
- `SUB` = `-`
- `MUL` = `*`
- `DIV` = `/`
- `MOD` = `%`
- `ADD_STR` = `+`, конкатенация строк

## Input
Получение данных из вне осуществляется с помощью:

    INPUT <type> <promt (не обязательно)>

Как и PUSH_CONST, она складывает в стек введенную `строку / число / булевое_значение`

    LABEL main
        INPUT int "Enter a number: "    ; ввод данных
        STORE_VAR num                   ; объявление переменной

        PUSH_CONST str "You entered: "  ; добавляем в стек
        LOAD_VAR num                    ; получаем значение и добавляем в стек
        ADD_STR                         ; складываем в строку (конкатенация строк)
        PRINT                           ; выводим

## JUMP
`JUMP <name>` - переводит в метку.
Используется с `LABEL`

    LABEL main
    
        JUMP print                      ; прыгаем к метке print

        LABEL print
            PUSH_CONST str "Hello, World!"
            PRINT

        HALT 0

## If-Else
`if-else` реализована с помощью `JUMP` и меток. А операции берут последние значения из стека

Операции:
- `EQ` = `==`
- `NEQ` = `!=`
- `LT` = `<`
- `GT` = `>`
- `LTE` = `<=`
- `GTE` = `>=`

Также `JUMP_IF_FALSE <name>` - срабатывает тогда когда последнее значение из стека была `false`

Пример:
    
    LABEL main

        INPUT int "Enter a number"      ; получаем число
        STORE_VAR a                     ; сохраняем в переенную

        LOAD_VAR a                      ; загружаем переменную в стек
        PUSH_CONST int 2                ; добавляем в стек 2
        MOD                             ; делаем операцию: a % 2, получив два последних значения из стека
        PUSH_CONST int 0                ; добавляем в стек 0
        EQ                              ; сравниваем значения
        JUMP_IF_FALSE else              ; если EQ выдало false прыгаем в блок else

        
        LOAD_VAR a                     
        PUSH_CONST str " is even"     
        ADD_STR                         
        PRINT                          
        JUMP end                       


        LABEL else
            LOAD_VAR a
            PUSH_CONST str " is odd"
            ADD_STR
            PRINT
            JUMP end

        
        LABEL end
        HALT 0

Логические операции по типу `and, or, not`, выполняются с помощью: `AND`, `OR`, `NOT`
Они тоже берут два последних значения из стека, кроме `NOT` он берет одно значение. Но они все работают только с `bool` значениями

## While, For, For-Each реализованы также через JUMP и LABEL
## Массивы
Команды:
- `ARRAY_NEW <name> <type> <size>` - создает массив с фиксированной длиной, где все значения `null`
- `ARRAY_GET <name>` - кладет в стек значение. Принимает в качестве индекса последнее значение стека, а в качестве массива ее имя
- `ARRAY_SET <name>` - вставляет в массив <name> значение из стека с индексом из стека
- `ARRAY_LENGTH <name>` - кладет в стек длину массива


    LABEL main
        
        ARRAY_NEW arr int 3             ; создаем массив
        
        PUSH_CONST int 1
        PUSH_CONST int 10
        ARRAY_SET arr                   ; вставляем в 1 индекс arr значение 10

        PUSH_CONST int 1
        ARRAY_GET arr                   ; получаем значение с индексом 1
        PRINT

        HALT 0

## Функции
Функции тоже реализованы через `JUMP` и `LABEL`. 
Вызвать функцию можно через `CALL <name>`.
Аргументы объявляются с помощью `STORE_VAR`
Передача аргументов осуществляется с помощью стека.
Функции возвращают последний элемент стека с помощью команды `RET`.

    LABEL add
        STORE_VAR a                     ; создаем аргументы
        STORE_VAR b                     ; еще один аргумен

        LOAD_VAR a                      ; загружаем в стек a
        LOAD_VAR b                      ; загружаем b
        ADD                             ; прибавляем их и кладем в стек

        RET                             ; возвращаем последнее значение из стека

    LABEL main
        INPUT int "Enter a first num: "
        INPUT int "Enter a second num: "
        CALL add
        STORE_VAR a
        
        PUSH_CONST str "Sum is: "
        LOAD_VAR a
        ADD_STR
        PRINT
        
        HALT 0
