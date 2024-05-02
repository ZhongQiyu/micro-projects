public class LeetCode {
    //credit to my fellow students' recommendations
    
    private int problemsSolved = 0;
    private double timeAvg = 0.0;
    private String todayDate = "";
    private ArrayList<String> readTutorials = new ArrayList<>();
    
    public LeetCode(int problemsSolved, double timeAvg) {
        this.problemsSolved = problemsSolved;
        this.timeAvg = timeAvg;
    }
    
    //1
    //题解
    //当相加的数小于两个时，直接返回null；是否需要考虑多于三个数的情况？
    //当相加的数等于两个时，使用嵌套for循环，遍历nums直到找到两数相加结果
    //等于target的indices，以array的形式return。
    //感觉可以不用递归然后降一下complexity
        
    //代码
    public int[] twoSum(int[] nums, int target) {
        int numCount = nums.length;
        if(numCount < 2) {
            return null;
        }
        else {
            int[] indices = new int[2];
            for(int i = 0; i < numCount - 1; i++) {
                int first = nums[i];
                for(int j = i + 1; j < numCount; j++) {
                    int second = nums[j];
                    if(first + second == target) {
                        indices[0] = i;
                        indices[1] = j;
                        return indices;
                    }
                }
            }
            return null;
        }
    }
    
    //2
    //题解
        
    //代码
    public ListNode addTwoNumbers(ListNode l1, ListNode l2) {
        if ((l1 == null) || (l2 == null)) {  // nodes that are not even existent
            return null;
        }
        else {
            ListNode currentSum = new ListNode(0);
            ListNode mainSum = currentSum;
            ListNode[] nodes = {l1, l2};
            int sum;
            int carry;
            boolean end = false;

            while(!end) {
                
                sum = nodes[0].val + nodes[1].val;
                carry = (int) (sum + currentSum.val) / 10;
                currentSum.val = (currentSum.val + sum) % 10;

                if ((nodes[0].next == null) && (nodes[1].next == null)) {
                    end = true;
                    // if there is an extra bit at the end, then carry
                    if (carry > 0) {
                        currentSum.next = new ListNode(carry);
                        currentSum = currentSum.next;
                    }
                }
                
                else {
                    if ((nodes[0].next == null) && (nodes[1].next != null)) {
                        nodes[0].next = new ListNode(0);
                    }
                    else if ((nodes[0].next != null) && (nodes[1].next == null)) {
                        nodes[1].next = new ListNode(0);
                    }
                    currentSum.next = new ListNode(carry);
                    currentSum = currentSum.next;
                    nodes[0] = nodes[0].next;
                    nodes[1] = nodes[1].next;
                }
            }
            return mainSum;
        }
    }
    
    //3
    //题解
        
    //代码
    public int lengthOfLongestSubstring(String s) {

    }
    
    //4
    //题解
        
    //代码
    /*
    A subroutine that truncates the original String,
    so that the processing of palindrome would be easier.
    */
    public StringBuilder truncate(String integerString) {
        StringBuilder truncated;
        int stringLength = integerString.length();
        if (stringLength % 2 == 0) {
            truncated = new StringBuilder(integerString);
        }
        else {
            truncated = new StringBuilder();
            int mid = stringLength / 2;
            for (int i = 0; i < stringLength; i++) {
                if (i != mid) truncated.append(integerString.charAt(i));
            }
        }
        return truncated;
    }

    /*
    A subroutine that turns an integer into a String,
    for determining its being whether a palindrome or
    not.
    */
    public boolean isPalindrome(int x) {
        String integerString = Integer.toString(x);
        integerString = new Solution().truncate(integerString).toString();
        int stringLength = integerString.length();
        for (int i = stringLength - 1; i >= 0; i--) {
            if (integerString.charAt(i) != integerString.charAt(stringLength - 1 - i)) {
                return false;
            }
        }
        return true;
    }

    /*
    进阶：你能不将整数转为字符串来解决这个问题吗？
    */
    
    //14
    //题解
    
    //代码
    public 
    
    //35
    //题解
        
    //代码
    public int binarySearch(int[] arr, int target) {
        int arrLength = arr.length;
        int index = arrLength / 2;
        int compared = 0;
        int maxSearchCount = Math.log(arrLength) / Math.log(2);
        int start = 0;
        int end = arrLength;
        boolean found = false;
        while ((!found) && (compared < maxSearchCount)) {
            if (arr[index] == target) {
                found = true;
            }
            else if (arr[index] > target) {  // meaning to search on the left branch of the current range (start, end)
                index = index - index / 2;
                end = index;
            }
            else {  // arr[index] < target, meaning to search on the right branch of the current range (start, end)
                index = index + index / 2;
                start = index;
            }
            compared++;
            // linear search
            for (int i = start; i < end; i++) {
                if (arr[index] == target) {
                    return index;
                }
            }
        }
        return index;
    }
    
    public static void main(String[] args){
        System.out.println("Welcome to the world of LeetCode");
        
        LeetCode runtime = new LeetCode();
        int[] nums = {1, 3, 5, 6};
        int target1 = 5;
        int target2 = 2;
        int target3 = 7;
        
        System.out.println(binarySearch(nums, target1));
        //System.out.println(binarySearch
    }
}
