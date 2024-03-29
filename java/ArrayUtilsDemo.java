/*
大家好，又见面了，我是你们的朋友全栈君。

java中删除 数组中的指定元素要如何来实现呢，如果各位对于这个算法不是很清楚可以和小编一起来看一篇关于java中删除 数组中的指定元素的例子。

java的api中，并没有提供删除数组中元素的方法。虽然数组是一个对象，不过并没有提供add()、remove()或查找元素的方法。这就是为什么类似ArrayList和HashSet受欢迎的原因。

不过，我们要感谢Apache Commons Utils，我们可以使用这个库的ArrayUtils类来轻易的删除数组中的元素。不过有一点需要注意，数组是在大小是固定的，这意味这我们删除元素后，并不会减少数组的大小。

所以，我们只能创建一个新的数组，然后使用System.arrayCopy()方法将剩下的元素拷贝到新的数组中。对于对象数组，我们还可以将数组转化为List，然后使用List提供的方法来删除对象，然后再将List转换为数组。

为了避免麻烦，我们使用第二种方法：

我们使用Apache commons库中的ArrayUtils类根据索引来删除我们指定的元素。

Apache commons lang3下载地址：

http://commons.apache.org/proper/commons-lang/download_lang.cgi

下载好后，导入jar。
*/

import java.util.Arrays;

import org.apache.commons.lang3.ArrayUtils;

/**

*

* Java program to show how to remove element from Array in Java

* This program shows How to use Apache Commons ArrayUtils to delete

* elements from primitive array.

*

*/

public class RemoveObjectFromArray{
 

public static void main(String args[]) {
 

//let’s create an array for demonstration purpose

int[] test = new int[] { 101, 102, 103, 104, 105};

System.out.println(“Original Array : size : ” test.length );

System.out.println(“Contents : ” Arrays.toString(test));

//let’s remove or delete an element from Array using Apache Commons ArrayUtils

test = ArrayUtils.remove(test, 2); //removing element at index 2

//Size of array must be 1 less than original array after deleting an element

System.out.println(“Size of array after removing an element : ” test.length);

System.out.println(“Content of Array after removing an object : “

Arrays.toString(test));

}

}

/*
Output:

Original Array : size : 5

Contents : [101, 102, 103, 104, 105]

Size of array after removing an element : 4

Content of Array after removing an object : [101, 102, 104, 105]

当然，我们还有其他的方法，不过使用已经的库或java api来实现，更快速。

我们来看下ArrayUtils.remove(int[] array, int index)

方法源代码：

public static int[] remove(int[] array, int index) {
 

return (int[])((int[])remove((Object)array, index));

}

在跳转到remove((Object)array, index)) ，源代码：

private static Object remove(Object array, int index) {
 

int length = getLength(array);

if(index >= 0 && index < length) {
 

Object result = Array.newInstance(array.getClass().getComponentType(), length – 1);

System.arraycopy(array, 0, result, 0, index);

if(index < length – 1) {
 

System.arraycopy(array, index 1, result, index, length – index – 1);

}

return result;

} else {
 

throw new IndexOutOfBoundsException(“Index: ” index “, Length: ” length);

}

}

这下明白了ArrayUtils的删除数组中元素的原理了吧。其实还是要用到两个数组，然后利用System.arraycopy()方法，将除了要删除的元素外的其他元素都拷贝到新的数组中，然后返回这个新的数组。

以上就是小编为大家带来的java中删除 数组中的指定元素方法全部内容了，希望大家多多支持脚本之家~

发布者：全栈程序员栈长，转载请注明出处：https://javaforall.cn/169512.html原文链接：https://javaforall.cn
*/