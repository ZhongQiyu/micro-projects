import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.DelayQueue;
import java.util.concurrent.RecursiveTask;

/*
ThreadPoolExecutorTest.java
A class that learns and implements the ThreadPoolExecutor that is embedded within Java.
*/

public class ThreadPoolExecutorTest {

	public static void main(String[] args) {
		ThreadPoolExecutor exec = new ThreadPoolExecutor(1, 1, 10, new TimeUnit(), new DelayQueue());
	}

}