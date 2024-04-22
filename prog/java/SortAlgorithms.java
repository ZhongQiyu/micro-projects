import java.util.ArrayList;

public class SortAlgorithms {

    ArrayList<String> algorithms;

    /*

     */
    public SortAlgorithms() {

    }

    /*

     */
    public SortAlgorithms(ArrayList<String> algorithms) {

    }

    /*

     */
    public static void bubbleSort(int[] arr) {
        int temp = 0;
        boolean swap;
        for (int i = arr.length - 1; i > 0; i--) { // 每次需要排序的长度
            swap=false;
            for (int j = 0; j < i; j++) { // 从第一个元素到第i个元素
                if (arr[j] > arr[j + 1]) {
                    temp = arr[j];
                    arr[j] = arr[j + 1];
                    arr[j + 1] = temp;
                    swap=true;
                }
            }//loop j
            if (swap==false){
                break;
            }
        }//loop i
    }// method bubbleSort

    public static void selectionSort(int[] arr) {
        int temp, min = 0;
        for (int i = 0; i < arr.length - 1; i++) {
            min = i;
            // 循环查找最小值
            for (int j = i + 1; j < arr.length; j++) {
                if (arr[min] > arr[j]) {
                    min = j;
                }
            }
            if (min != i) {
                temp = arr[i];
                arr[i] = arr[min];
                arr[min] = temp;
            }
        }
    }

    public static void insertionSort(int[] arr){
        for (int i=1; i<arr.length; ++i){
            int value = arr[i];
            int position=i;
            while (position>0 && arr[position-1]>value){
                arr[position] = arr[position-1];
                position--;
            }
            arr[position] = value;
        }//loop i
    }

    public static void mergeSort(int[] arr){
        int[] temp =new int[arr.length];
        internalMergeSort(arr, temp, 0, arr.length-1);
    }
    private static void internalMergeSort(int[] arr, int[] temp, int left, int right){
        //当left==right的时，已经不需要再划分了
        if (left<right){
            int middle = (left+right)/2;
            internalMergeSort(arr, temp, left, middle);          //左子数组
            internalMergeSort(arr, temp, middle+1, right);       //右子数组
            mergeSortedArray(arr, temp, left, middle, right);    //合并两个子数组
        }
    }
    // 合并两个有序子序列
    private static void mergeSortedArray(int arr[], int temp[], int left, int middle, int right){
        int i=left;
        int j=middle+1;
        int k=0;
        while (i<=middle && j<=right){
            temp[k++] = arr[i] <= arr[j] ? arr[i++] : arr[j++];
        }
        while (i <=middle){
            temp[k++] = arr[i++];
        }
        while ( j<=right){
            temp[k++] = arr[j++];
        }
        //把数据复制回原数组
        for (i=0; i<k; ++i){
            arr[left+i] = temp[i];
        }
    }

    public static void quickSort(int[] arr){
        qsort(arr, 0, arr.length-1);
    }
    private static void qsort(int[] arr, int low, int high){
        if (low >= high)
            return;
        int pivot = partition(arr, low, high);        //将数组分为两部分
        qsort(arr, low, pivot-1);                   //递归排序左子数组
        qsort(arr, pivot+1, high);                  //递归排序右子数组
    }
    private static int partition(int[] arr, int low, int high){
        int pivot = arr[low];     //基准
        while (low < high){
            while (low < high && arr[high] >= pivot) --high;
            arr[low]=arr[high];             //交换比基准大的记录到左端
            while (low < high && arr[low] <= pivot) ++low;
            arr[high] = arr[low];           //交换比基准小的记录到右端
        }
        //扫描完成，基准到位
        arr[low] = pivot;
        //返回的是基准的位置
        return low;
    }



}
