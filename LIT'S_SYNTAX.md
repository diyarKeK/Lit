# Синтаксис Lit

### 1. Создание первой программы
Чтобы сделать первую программу на `Lit`:
- a. Создайте файл с расширением `.lit`
- b. Напишите туда:

      fun main() {

      }

Функция `main()` - это точка входа в вашу программу. 
Слово `fun` обозначает что мы объявляем функцию.

### 2. Вывод текста в консоль
Чтобы вывести текст в консоль, используйте функцию `print()`:

    fun main() {
        print('Hello, World!')
    }

Как мы видим, чтобы обозначить строку, нужно использовать 
одинарные кавычки `''`. Также мы видим что нам 
вообще не нужно писать `;` в конце строчки

### 3. Переменные
В Lit существует только 4 стандартных типа переменных:
- a. `unt` - положительное целочисленное число
- b. `int` - целочисленное число, которое может содержать и отрицательные числа
- c. `float` - число с плавающей точкой
- d. `bool` - булевое значение
- e. `str` - строка

Чтобы объявить переменную, укажите тип переменной, 
дальше ее имя, потом знак присвоения `=`, 
а потом уже ее значение:

    fun main() {
        unt a = 10
        int b = -1
        float c = 2.5
        bool ok = true
        str greeting = 'Hello, World'
    }

### 4. Комментарии
Чтобы поставить комментарий в коде, укажите `//` 
перед комментарием:

    fun main() {
        // Комментарий
        // Еще Комментарий
        // И Еще Комментарий
        /*
        Многострочный Комментарий
        */
    }

### 5. if-else
Чтобы сделать `if-else` конструкцию, укажите 
ключевое слово `if`, затем условие, 
затем откройте фигурные скобки и напишите 
внутри нее тело конструкции, которые выполняться 
если `if` истина. Запишите `else if` для следующего 
условия или `else` если все условия для исключения

В условиях `if-else` конструкций (да и в условиях while 
циклов конечно же) можно использовать:

- a. `and` - логическое `И`
- b. `or` - логическое `ИЛИ`
- c. `not` - логическое `НЕ`
- d. `>` - больше
- e. `<` - меньше
- f. `>=` - больше или равно
- g. `<=` - меньше или равно
- h. `==` - равно
- i. `!=` - не равняется

#### Практика:

    fun main() {
        int a = 10
        float b = 8.5
        bool c = true
        str d = 'Hello, World'
        
        if a < 10 {
            print('A is less than 10!')
        } else if b == 3.14 {
            print('B equals to PI')
        } else if not c {
            print('minus to minus gives plus')
        } else if s == 'Coffee' {
            print('Do You Want Some Coffee?')
        } else {
            print('Nothing to show you')
        }

    // Также если body if конструкции имеет только одну строчку кода можно сократить до:
        if a < 10 
            print('1 Condition')
        else if b == 3.14 
            print('2 Condition')
        else if not c 
            print('3 Condition')
        else if s == 'Coffee' 
            print('4 Condition')
        else 
            print('Else Block')
    }

### 6. Очистка памяти
У `Lit` есть свой сборщик мусора (`GC`), чтобы программисты не морочили голову над очисткой памяти

### 7. Работа с input()
Чтобы принять данные из вне, 
достаточно написать переменную и 
указать `input()` как ее значение:

    fun main() {
        str s = input('Введите текст: ') // Можно написать что выведется в консоль
        print('Вы ввели: {s}')
    }

### 8. Циклы while и for
В Lit есть 2 типа циклов:
- a. `while`
- b. `for`

Чтобы создать цикл, напишите условие при котором оно 
будет выполняться, а затем тело самого цикла:
    
    fun main() {
        int i = 0
        while i < 10 {
            print(i)
            i++
        }
        
        for int j = 0, j < i + 10, j++ {
            print(j)
        }
    }

Чтобы выйти из цикла, используйте 
`break`, а чтобы принудительно 
перейти к следующей итерации 
используйте `continue`:

    fun main() {
        int i = 0
        while i < 5 {
            if i == 4 
                break
            i++
        }
      
        for int j = 0, j < 13, j++ {
            if j % 2 == 0 
                continue
            else if j + 1 == 7 
                break
        }
    }

Также циклы поддерживают конструкцию
`else`, и она 
сработает тогда когда цикл завершен 
полностью, без принудительного 
`break`:

    fun main() {
        while true {
            break
        } else {
            print('End of the cycle')
        }
        // оператор 'break' полностью выходит из цикла, что даже 'else' не срабатывает
        
        for int i = 0, i < 10, i++ {
            print('I: {i}')
        } else {
            print('Cycle has been finished')
        }
    }

### 9. Массивы
Чтобы создать массив, укажите 
сначала тип переменной, потом квадратные скобки
до названия массива:
    
    fun main() {
        // количество элементов в массиве не изменяемо

        int[] array = [1, 2, 3] // сам определяет максимальное количество элементов в массиве
        int[] new_array = new int[3] // в массиве может быть только 3 элемента
        
        // Работа с массивами

        // Нам не нужно уточнять каким будет переменная el. Мы просто ее объявлеяем
        for el in array {
            print(el)
        }
      
        // можно использовать length:
        for unt i = 0, i < array.length, i++ {
            print(array[i])
        }
    }


### 10. Функции
Чтобы сделать функцию, укажите 
ключевое слово `fun`, затем имя 
функции, дальше открываете круглые
скобки, и пишите туда аргументы:

    fun main() {
        int a = add(3, 2)
        do_something()
    }
    
    // Используйте return чтобы вернуть значение

    fun add(int a, int b): int { // возвращает int
        return a + b
    } 
      
    fun do_something() { // ничего не возвращает
        print('Hello')
    }
    fun sort(int[] array): int[] {
        for int i = 0, i < array.length - 1, i++ {
            for int j = 0, j < array.length - i - 1, j++ {
                if array[j] > array[j + 1] {
                    int temp = array[j]
                    array[j] = array[j + 1]
                    array[j + 1] = temp
                }
            }
        }
        return array
    }

### 11. Лямбда выражения
Чтобы создать лямбда выражения, 
используйте ключевое слово `lambda` 
перед объявлением имени выражению:

    fun main() {
        lambda add = (int a, int b) => int {
            return a + b
        }
    
        lambda do_something = () => {
            print('Hello')
        }
    
        print(add(3, 2)) // 5
        do_something()
    }


### 12. Switch-case
Чтобы сделать `Switch-case` конструкцию 
укажите ключевое слово switch, дальше
переменную которую будем проверять, 
дальше `case'ы`:

    fun main() {
        str name = 'Alex'
        switch name {
            'David' => print('Name is David')
            'Alex' => print('Name is Alex')
            default => print('Name is Unknown')
        }
    }

В функциях где нужно возвращать:

    fun main() {
        int id = getIdByCase('Alex')
        print(id)
    }
    
    fun getIdByCase(str name): int {
        return switch name {
            'David' => 1
            'Alex' => 2
            default => 0 // Обязательно default
        }
    }

### 13. Импорты
Чтобы импортировать какой либо класс, 
используйте ключевое слово `import` 
в файле, дальше напишите её `package`:

    import util.Math
    
    fun main() {
        print(Math.sin(30))
    }

Также можно импортировать класс как, словом `as`:

    import util.Math as M
    // Теперь класс Math импортирован как M
    
    fun main() {
        print(M.cos(30))
    }

Можно добавлять к классу ее `package`.

    package main

    fun main() {
        print('This file's package is: main')
    }

### 14. try-catch и throw
Для написания `try-catch` конструкции, 
укажите ключевое слово `try`, дальше
в фигурных скобках тело выполнения, 
затем `catch` и название `Exception`, 
потом в фигурных скобках код, который
сработает при ошибке, например:
`print(err.getMessage())`:

    import lang.Int
    import lang.NumberFormatException as NFE
    
    fun main() {
        try {
            int a = Int.to_int('123')
            print(a)
            int b = Int.toInt('s') // Ошибка в рантайме

        } catch NFE err {

            print(err.getMessage())
        }
    }

Чтобы выйти из программы с ошибкой,
укажите ключевое слово `throw` дальше
`new` и затем `Exception`, например:

    import lang.Strings as Str
    import lang.RuntimeException as RE
    
    fun main() {
        str str_num = input('Введите число: ')

        if not Str.is_digit(str_num) {
            throw new RE('Not Digit!') // и выходим автоматически из программы, так как написан throw
        } else {
            ...
        }
    }

Если что `try-catch` конструкции блокируют 
throw. А если и произошла ошибка срабатывается код из `catch`.

### 15. Дефолтные значения в функциях
В функциях можно писать дефолтные 
значения переменным, через знак `=`:
    
    import util.Math
    import lang.RuntimeException as RE
    
    fun main() {
        float res1 = calculate(2, 5) // 7
        float res2 = calculate(2, op='*', 5) // 10
    }
    
    fun calculate(float a, str op='+', float b): float {
        if op == '/' and b == 0 
            return -1.0
        
        return switch op {
            '+' -> a + b
            '-' -> a - b
            '*' -> a * b
            '/' -> a / b
            '%' -> a % b
            '^' -> Math.pow(a, b)
            default -> throw new RE('Unknown operation: {op}')
        }
    }

Фан-факт: 
функция `print()` выглядит так:

    fun print(str s, str end='\n') { /* built-in */ }

### 16. Написание множества строк кода в одной строке
Чтобы такое провернуть нужно 
разделять их запятой:

    fun main() {
        print('Hi, ', end=''),  print('Alex', end=''),  print('!')
    }

### 17. Классы
Чтобы создать класс, укажите ключевое 
слово `class` дальше ее название:
    
    class MyClass {
        ...
    }

Чтобы создать конструктор класса, 
просто укажите имя класса, затем в 
скобках укажите параметры, которые 
принимают класс:

    class MyClass {
        str name
      
        MyClass(str name) {
            this.name = name // this - означает что мы обращаемся к полю экземпляра
        }
    }

Чтобы поле могли видеть вне класса, 
укажите ключевое слово `gl` перед переменной:

    class MyClass {
        gl str name
      
        MyClass(str name) {
            this.name = name
        }
    }

Чтобы создать не изменяемое поле, 
укажите ключевое слово `final`:
    
    class MyClass {

        gl final str name
        ...
    }

### 18. Использование класса:

    class User {
        gl final int id
        gl str name
        
        User(str name) {
            this.id = count
            this.name = name
        }
    }

    fun main() {
        // Чтобы сделать объект класса, укажите ключевое слово 'new':
        User example = new User('Mark')
    }

### 19. Защита от NullPointer
Как мы знаем, экземпляры классов 
могут быть `null`. Чтобы защитится от 
`null`, можно указать, что `null` будет 
заменяться на дефолтное значение:

    gl null = (дефолтное значение)

Повторюсь это опционально. Можно 
не писать эту строчку кода, если не
хотите защиты от `NullPointer`
    
    class User {
        gl str name
        
        gl null = new User('Guest')
      
        User(str name) {
            this.name = name
        }
    }
    
    fun main() {
        User user = null
        print(user.name) // Guest
      
        // можно также его использовать:
        print(User.null.name) // Guest
    }

### 20. abstract, interface, data, enum, exception классы
Чтобы объявить `abstract` класс, 
укажите ключевое слово `abstract`:
    
    import pc.accessories.ProcessorType
    
    abstract PC {
        gl ProcessorType type
      
        ...
      
        fun turn_on() {
            ...
        }
      
        abstract fun execute()
    }

    // Чтобы указать что класс наследуется укажите ':' (двоеточие)
    class AMD_PC : PC {
        ...
    }

Ключевым словом `interface`, 
создаются `interface` классы:
    
    interface Car {
      fun drive()
    }
    
    // Укажите `:` (двоеточие), чтобы класс реализовывал interface
    class BMV : Car {
      fun drive() {
        ...
      }
    }
---
### Обычные классы не наследуются

---
`Data` классы хранят только переменные, 
никакой реализации.
Чтобы объявить `data` класс, укажите 
ключевое слово `data`, дальше имя 
класса, а потом в обычных скобках
переменные которые будет хранить 
класс:

    data Product(str name, float price)

Чтобы сделать класс перечисления, 
укажите ключевое слово `enum`:

    enum Weapon {
        SWORD,
        BOW
    }
    
    enum Role {
        USER,
        VIP,
        ADMIN
    }

`Exception` классы, это классы 
исключения, срабатывают при ошибках
и тд:

    exception MyException {
        str desctiption
    
        MyException(str description) {
            this.description = description
        }
      
        fun getMessage(): str {
            return description
        }
        ...
    }
    
    // Чтобы использовать exception нужно
    // указать ключевое слово 'throw':
    
    fun main() {

        throw new MyException('Something is Wrong') 
    }
Выведется: Error At *.lit:18: Something is Wrong

### 21. instance_of
Ключевое слово `instance_of` возвращает
булевое значение, и определяет,
был ли заданный объект, объектом 
заданного класса.

На примере игры:

    import game.entity.Entity
    import game.item.Item
    import game.particle.Particle
    
    class ItemEntity : Entity {
        gl final Item item
      
        ItemEntity(Item item) {
            this.item = item
      
            summon()
        }
      
        fun summon() {
            ...
        }
        
        fun take() {
            ...
        }
    }
    
    class ParticleEntity : Entity {
        gl final Particle particle
        gl final int time
      
        ParticleEntity(Particle particle, int time) {
            this.particle = particle
            this.time = time
      
            summon()
        }
      
        fun summon() {
            ...
        }
      
        fun destroy() {
            ...
        }
    }
    
    fun main() {
        Particle particle = new Particle(...)
        Entity entity = new ParticleEntity(particle, 1)

        if entity instance_of ItemEntity { // false

          ...
        } else if entity instance_of ParticleEntity { // true

          ...
        }
    }

### 22. Дженерики
Итак, чтобы принять `дженерики` в класс,
после названия класса открываем 
`< >`, и дальше туда пишем название 
для `дженериков`.
Вы можете принимать любое количество 
`дженериков` в свой класс и помечать их как угодно:

    class MyList<E> { // принимаем дженерик, пометив его как E (сокращение от слово Element) для наглядности 
        E[] current_array
    
        MyList() {
            // создаем массив из типа принятого дженерика
            current_array = new E[1]
        }
      
        MyList(int initial_size) {
            current_array = new E[initial_size]
        }
        ...
        
        fun add(E e) { // принимаем в качестве типа аргумента принятый дженерик
            E[] new_array = new E[current_array + 1]
            
            for int i = 0, i < new_array.length, i++ {
                if i == new_array.length - 1 {
                    new_array[i] = e
                    break
                }
                new_array[i] = current_array[i]
            }
        
            current_array = new_array
        } 
      
        fun sort(lambda: (E[] array) => E[] method) {
            current_array = method(current_array)
        }
      
        fun size(): int {
            return current_array.length
        }
      
        fun get(int index): E { // Возвращаем с типом принятого дженерика
            if index < 0 or index >= current_array.length 
                return null
            return current_array[index]
        }
        ...
    }
    
    // Еще пример:
    
    class MyMap<K, V> { // принимаем два дженерика: например, где K, класс будет ключем, где V, значением
        ...
        MyMap() {
            ...
        }
      
        fun put(K key, V value) {
            ...
        }

        fun get(K key): V {
            ...
        }
    }
    
    fun main() {
        MyList<str> list = new MyList();
        // даем примитивный класс str
      
        MyList<str[]> list_of_arrays = new MyList(10)
        // создаем список из массивов str с начальной ёмкостью 10 
      
        MyMap<int, MyList<str>> map = new MyMap();
        // даем в качестве ключа класс int, а в качестве значения полноценный класс MyList
    }

### 23. Асинхронность
В Lit поддерживаются два вида асинхронного программирования:
  - a. `Корутины` (через ключевое слово `launch`)
  - b. `async/await` (для получения результата асинхронной функции)

  #### 1. Корутины:

Корутины — это лёгкие параллельные задачи, которые не блокируют основной поток.  
Они запускаются с помощью `launch { ... }` внутри обычной функции.

#### Пример:

    fun count(int until) {
        launch {
            for int i = 1, i <= until, i++ {
                print('Count: {i}', end=', ')
            }
        }
    }
    
    fun main() {
        count(5)
        print('Main завершилась')
    }
    
    // Примерный вывод:
    // Main завершилась
    // Count: 1, Count: 2, Count: 3, Count: 4, Count: 5,

  #### 2. async / await 

Если нужно дождаться результата асинхронной функции, используется `async` и `await`.

Асинхронная функция объявляется с помощью `async fun`, а внутри неё можно использовать `await` чтобы подождать другую асинхронную операцию.

#### Пример:

    import net.http.HttpMethod
    import net.http.Response
    
    async fun fetch_data(): str {
        Response response = await fetch('https://example.com/data', HttpMethod.GET)
        delay(1000)
        return 'Data has been uploaded'
    }
    
    fun main() {
        str res = fetch_data()
        print(res)
        print('End')
    }
    
    
    // Примерный вывод:
    // End
    // Data has been uploaded

Поведение async-функций:

Вызов `fetch_data()` сразу запускает функцию, но НЕ блокирует `main()`.  
Поэтому `End` выводится раньше, чем результат `fetch_data()`.

Чтобы дождаться результата внутри основной функции, используйте `await`, но только внутри async-функции:

    async fun main() {
        str res = await fetch_data()
        print(res)
        print('End')
    }

    // Тогда:
    // 1. Data has been uploaded
    // 2. End

### 24. Принятие в качестве аргументов лямбда выражения
Чтобы принять лямбду выражения, 
достаточно написать какие аргументы
лямбда выражение будет принимать и 
что возвращать:

    fun accept_lambda(lambda: (int a, int b) => int some, int c, int d): int {
        return some(c, d)
    }
    
    fun main() {
        int res = accept_lambda((a, b) => { // Можно как угодно называть a и b, так как типы a и b уже определены в лямбде выражения some
            return a * a + 2 * a * b + b * b
        }, 2, 3)
      
        print(res) // 25
    }

### 25. Модули
Модуль - это совокупность функций. Это подходит для построения Утилит.<br>
Чтобы создать модуль, достаточно перед ее именем поставить ключевое слово `module`:

    package src

    module MyUtils {

        fun pow(float a, int b): float {
            float result = 1.0

            for int i = 0, i < abs(b), i++
                result *= a

            if b < 0
                return 1.0 / result
            return result
        }

        fun abs(float a): float {
            if a < 0 
                return -a
            return a
        }
    }

Модули импортируются также, как и обычные классы:

    import path.to.module.Module

Использование модуля:

    import src.MyUtils

    fun main() {
        int a = 10
        int b = 3

        int result = MyUtils.pow(a, b)
        print(result)
    }

## Системные Функции
  - a. `print(str s, str end='\n')` - вывод на экран
  - b. `input(str s)` - принятие данных
  - c. `len(str s)` - возвращает длину строки
  - d. `char_at(str s, int index)` - возвращает символ с индекса строки
  - e. `warn(str s)` - как и `print` она выводит в консоль текст, но с желтым цветом, для предупреждения
  - f. `open(str path, str act)` - открывает файл для записи `act == 'w'` или ее чтения `act == 'r'`, возвращает текст файла даже при записи
  - g. `delay(int millis)` - останавливает корутину или поток, из которого он запущен, на количество миллисекунд
  - h. `fetch(str url, str method, JSON body = null)` - отправляет по `url` наш запрос метод `method`, и по умолчанию она не имеет `body` так как `method` не всегда может быть `POST/PUT`
  - i. `exit(int code, str msg='')` - выходит из программы принудительно. В конце может показать сообщение если вы его ввели.

## Список ключевых слов
Возможно тут могут быть не все ключевые слова
  - a. `unt`
  - b. `int` 
  - c. `float`
  - d. `bool`
  - e. `str`
  - f. `if`
  - g. `else if`
  - h. `else`
  - i. `and`
  - j. `or`
  - k. `not`
  - l. `while`
  - m. `for`
  - n. `break`
  - o. `continue`
  - p. `in`
  - q. `fun`
  - r. `return`
  - s. `lambda`
  - t. `switch`
  - u. `default`
  - v. `import`
  - w. `as`
  - x. `package`
  - y. `try`
  - z. `catch`
  - aa. `throw`
  - ab. `class`
  - ac. `this`
  - ad. `gl`
  - ae. `final`
  - af. `new`
  - ag. `null`
  - ah. `data`
  - ai. `exception`
  - aj. `abstract`
  - ak. `interface`
  - al. `enum`
  - am. `instance_of`
  - an. `launch`
  - ao. `async`
  - ap. `await`
  - aq. `module`

## Приколы
  - a. Если человек сделал программу 
`Hello World` на `Lit`, то компилятор 
поймет это и заменит
`print('Hello World!')` на 
`print('Hello World is not enabled in Lit! :)')`
