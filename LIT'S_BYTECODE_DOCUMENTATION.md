# Демонстрация байт-кода LVM

## 1. Первая программа для LVM:
### Для того чтобы начать, нужно создать файл с расширением: `.lbc`

Чтобы создать программу на `.lbc`,
нужно обозначить точку входа в программу.
Она обозначается как: `label main`

    label main:
        
        ; Your Program

        halt 0

Комментарии в .lbc обозначаются через `;`

Команда `halt (<exit_code>)` выходит из программы

## 2. Вывод в консоль
Чтобы вывести в консоль значение, перед этим, 
нам нужно поместить в стек само значение через
`push_cont <type> <val>`.

Дальше мы выводим значение через `print <type>`. 
Обязательно указывайте аргумент type в print. 
Это сделано из-за того что, стек может содержать только
64 битное без-знаковое число. И поэтому если не указать 
тип, то вывелось бы ее адрес в куче.

    label main:
        
        push_const str "Hello, World!"
        print str
        
        halt 0

## 3. Типы данных
Есть такие типы как:
- a. `unt` - беззнаковое 64 битное число.
- b. `int` - знаковое 64 битное число.
- c. `float` - 64 битное число с плавающей точкой.
- d. `str` - строка.
- e. `lambda` - лямбда выражение.

И чтобы ввести их в стек используем `push_const`:

    label main:

        push_const unt 1
        print unt

        push_const int 2
        print int

        push_const float 3.14
        print float
    
        push_const str "Alex"
        print str

        halt 0

На `lambda` тип пока что не зацикливаемся

## 4. Переменные
Чтобы сохранить значение из стека в переменную
используйте команду: `store_var <name>`.
А чтобы использовать переменную: `load_var <name>`:

    label main:
        
        push_const unt 14
        store_var x

        load_var x
        print unt

        halt 0

Нам не нужно писать каким будет тип переменной,
ведь это уже определено в `push_const`

## 5. IO
Чтобы получить данные из вне, используйте
команду `input <required_type>`:

    label main:
        
        push_const str "Enter a Number: "
        print str

        input int
        store_var a

        push_const str "a is: "
        print str

        load_var a
        print int

        halt 0

## 6. Арифметика
Команды для арифметика различаются по типам:
- a. `unt`
- b. `int`
- c. `float`

Всего 15 команд на арифметику:
- 
- Для `unt` - `u_add, u_sub, u_mul, u_div, u_mod`
- Для `int` - `i_add, i_sub, i_mul, i_div, i_mod`
- Для `float` - `f_add, f_sub, f_mul, f_div, f_mod`

Где суффикс `add` - сложение, `sub` - вычитание,
`mul` - умножение, `div` - деление, `mod` - 
остаток от деления.

    label main:

        push_const unt 4
        push_const unt 7
        u_mul
        store_var x

        ; 28
        load_var x
        print unt

        halt 0

## Сравнения и Jump
Для каждых типов, сравнение, как и арифметика, 
происходит по разному:
- a. Для `unt` - `u_eq, u_neq, u_lt, u_gt, u_lte, u_gte`
- b. Для `int` - `i_eq, i_neq, i_lt, i_gt, i_lte, i_gte`
- c. Для `float` - `f_eq, f_neq, f_lt, f_gt, f_lte, f_gte`

Суффик `eq` означает - равенство, `neq` - не равенство,
`lt` - меньше, `gt` - больше, `lte` - меньше или равно,
`gte` - больше или равно.


## Список всех команд в LVM:
- a. `label <name>`
- b. `halt (<exit_code>)`
- c. `push_const <type> <val>`
- d. `print <type>`
- e. `store_var <name>`
- f. `load_var <name>`
- g. `input <required_type>`
- h. `u_add`
- i. `u_sub`
- j. `u_mul`
- k. `u_div`
- l. `u_mod`
- m. `i_add`
- n. `i_sub`
- o. `i_mul`
- p. `i_div`
- q. `i_mod`
- r. `f_add`
- s. `f_sub`
- t. `f_mul`
- u. `f_div`
- v. `f_mod`
- w. `u_eq`
- x. `u_neq`
- y. `u_lt`
- z. `u_gt`
- aa. `u_lte`
- ab. `u_gte`
- ac. `i_eq`
- ad. `i_neq`
- ae. `i_lt`
- af. `i_gt`
- ag. `i_lte`
- ah. `i_gte`
- ai. `f_eq`
- aj. `f_neq`
- ak. `f_lt`
- al. `f_gt`
- am. `f_lte`
- an. `f_gte`