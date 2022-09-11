
# What is Chip?

Chip is designed to make C++ programmers feel at home as much as possible.
However, it is:
  * easier to parse than C++ `the fn keyword is popular in modern languages for a reason!`
  * cleaner than C++ (if/for/while don't need parenthesis, for instance)
  * less ambiguous than C++ (if/for/while always need to specify a block with braces)

Chip is:
* imperative
* functional
* object oriented
* strongly typed

Chip is not:
* designed with safety as the primary concern. Languages such as Rust are great for this
* trying to be simple. Chip is a multi-paradigm language, with many features.
* 


# What does Chip look like?

## functions
* functions are declared with the `fn` keyword
* parameters are declared with the form `name as type`
* parameters are separated with commas
* the contents of the function are placed between braces
* the return type is specified after the parenthesised argument list, preceded by `->`

```Rust
fn myFunc(name as string) -> void {
    // print() will output text to the screen.
    // println() will also output a newline at the end.
    println("hello , {0}!", name);

    // the return statement in a void function is optional,
    // but shown here to introduce the return keyword
    return;
}

// return type is optional! it will be deduced wherever possible
fn main() {
    myFunc("World");
}
```

## structures
just like other C-like languages, you can declare structures  
* this is done using the `class` keyword
* structures can contain fields and methods

you declare a field using the form `name as type;`  

## methods
methods are declared just like normal functions,  
although extra attributes can be inserted just before the `->`

```Rust
class Base {
    fieldA as i32;
    fieldB as i32;

    : new() {
        // a default constructor
    }

    // a method. there is an implicit 'this' variable
    // that refers to the structure instance, and can
    // also be referred to explicitly
    fn myMethod() -> void {
        println(fieldA);
        println(fieldB);
    }
}

//out-of-line function declaration
fn Base: someFunc() -> void {
    println(fieldA);
}

//out-of-line constructor declaration
Base: new(param as i32) {

}

//we can even do this to primitive types!
fn i32: add(other as i32) -> i32 {
    // this is of type i32*, so dereferencing
    // it will produce an i32
    return *this + other;
}

//we can make static methods too!
fn i32: square(other as i32) static -> i32 {
    return other * other;
}

class Derived -> Base {
    x as i32 = 2;
    y as i32 = 3;
    z as static i32 = 10;

    : new(_x as i32, y as i32) {
        // a constructor with parameters
        // initialize x
        x = _x;

        // parameters shadow fields, so 'this' must be
        // used to initialize y here
        this.y = y;
    }
    : delete() {
        //called when the object is destroyed
    }    

    // a static method. there is no 'this' variable
    // and this method cannot refer to non-static fields
    fn myStaticMethod() static -> void {
        println(z); // only z is visible from here
    }
}

fn main() {
    //implicitly construct MyStructure into myThing
    //possible because it has a default constructor
    //type of myThing is coerced to Base
    var myThing as Base;

    //ERROR: no default constructor defined for Derived
    var doesntWork as Derived;

    //explicitly construct MyStructure into myThing
    //type of foo is coerced to Derived
    var foo = Derived(5, 3);

    //explicitly construct myThing onto heap
    //type of bar is coerced to Derived*
    var bar = new Derived(5, 3);

    //explicitly construct myThing onto heap
    //type of baz is set to Base*, but holds a Derived
    var baz as Base* = new Derived()

    //free the memory pointed to by bar and baz
    delete bar;
    delete baz;
}
```

## generics
generics are compile-time only, like C++ templates  
they are specified with angle brackets, and a list of  
types separated by comms

```Rust
fn myGenericFunction<T, U>(T input) -> U {
    println(input);
    var output as U;
    return output
}
```
## branches
branches are created using the `if` and `else`
parenthesis aren't used, but braces are always required
```Rust

var x as i32 = 7;

if x > 5
{
    println("x is bigger than 5");
}
else
{
    println("is is small!";
}
```

## while loop
just like branches, no parenthesis are used, but it  
must always defined a block with braces

```Rust

var x as i32 = 0;

while x < 5 {
    println("x is currently {0}", x);
    x++
}
```

## for loops
for loops have a few forms:

* "standard" form
* ranged integer form
* ranged form

```Rust
fn myFunc(length as i32) -> void
{
    var x as i32;
    var y as i32;

    setCoords(&x, &y); // pass by reference

    // standard for loop. is able to specify
    // stride, and other creative things
    for var i = 0 : i < length : i += 2
    {
        doSomeCrap(i);
    }

    // ranged integer for loop
    // always has a stride of +1, and assumes the 
    // right hand side is greater than the left hand side
    // to iterate backwards, use the standard for loop above
    for var i : 0..length
    {
        // the 'as' casts the result of someFunction to an i32
        var something = someFunction(x, y, i) as i32;
    }

    // i like the idea of this, but i'm not entirely sure of the behaviour
    // when the third expression *subtracts* from i or does something weirder
    for var i : 0..length : i += 2
    {
        floopydoop(i);
    }

    // ranged for loop
    // you can also use for to iterate a ranged collection 
    for var item : collection
    {
        item.doSomething();
    }

}
```

## closures

TBD

## pattern matching

TBD

matches must be exhaustive.  
that is, every possible case of the match expression  
must be covered by a match arm.

```Rust
var x as i32;

match x {
    // single expression
    0 => println("x is 0");
    
    // single expression, multiple matches to one arm
    1, 2 => println("x is 1");

    // block expression
    3 => {
        println("x is 0");
    }

    //all the remaining match arms
    _ => println("x is 0");
}


```


TBD

## tagged unions

TBD