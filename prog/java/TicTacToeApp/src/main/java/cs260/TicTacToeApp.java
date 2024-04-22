package cs260;

import javax.swing.JFrame;


import cs260.game_model.TicTacToeGame;

import cs260.gui.FancyDisplay;

import cs260.turn_gui.TurnIndicator;


public class TicTacToeApp
{
    public static void main(String [] args)
    {
        TicTacToeApp main = new TicTacToeApp();
        main.start();
    }

    public void start()
    {
        JFrame mainFrame = new JFrame("Tic Tac Toe");
        TicTacToeGame game = new TicTacToeGame('X');

        FancyDisplay gameDisplay = new FancyDisplay(game);
		game.addListener(gameDisplay);

        mainFrame.getContentPane().add(gameDisplay);
        mainFrame.pack();
        mainFrame.setVisible(true);

        JFrame turnFrame = new JFrame("Tic Tac Toe");
        TurnIndicator indicator = new TurnIndicator(game);
		game.addListener(indicator);
        
        turnFrame.getContentPane().add(indicator);
        turnFrame.pack();
        turnFrame.setVisible(true);
    }
    
}
