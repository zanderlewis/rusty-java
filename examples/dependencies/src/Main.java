import com.google.common.base.Joiner;
import com.google.common.collect.Lists;

import java.util.List;

public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, world!");

        List<String> fruits = Lists.newArrayList("apple", null, "banana", "cherry");
        Joiner joiner = Joiner.on(", ").skipNulls();
        String result = joiner.join(fruits);
        System.out.println(result); // Output: apple, banana, cherry
    }
}