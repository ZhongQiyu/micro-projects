package sample;

import javafx.application.Application;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.stage.Stage;

public class Main extends Application {

    @Override
    public void start(Stage primaryStage) throws Exception{
        Parent root = FXMLLoader.load(getClass().getResource("sample.fxml"));
        primaryStage.setTitle("Hello World");
        primaryStage.setScene(new Scene(root, 300, 275));
        primaryStage.show();
    }

    /*
    Attempt to transform a number into bits.
    */
    private int toBits(int num) {
        if (num == 0) return 0;
        else {
            int remainder = 0;
            ArrayList<Integer> bits = new ArrayList<>();
            if (remainder == 0) {
                bits.add(1);
                int count = (int)Math.pow(num,1/2)+1;
                int index;
                for (index = 0; index < count; index++) bits.add(0);
            }
            else {
                int current = num;
                int bit = 0;
                while (current > 0) {
                    remainder = current%2;
                    
                    bit = ((remainder==0)||((remainder==1)&&(current/2>=1)))?1:0;
                    System.out.println("current: " + current + " bit: " + bit);
                    bits.add(bit);
                    current = current / 2;
                }
                return 0;
            }
        }
    }

    public static void main(String[] args) {
        launch(args);
    }
}
