package cs260.gui;

import java.awt.event.MouseListener;
import java.awt.event.MouseEvent;

import javax.swing.JOptionPane;

import cs260.game_model.TicTacToeGame;
import cs260.game_model.IllegalMoveException;

public class TicTacToeControl
        implements MouseListener
{
    private TicTacToeGame game;
    private FancyDisplay gameDisplay;
    
    public TicTacToeControl(TicTacToeGame game, FancyDisplay gameDisplay)
    {
        this.game = game;
        this.gameDisplay = gameDisplay;

        gameDisplay.addMouseListener(this);
    }

    public void mouseClicked(MouseEvent e)
    {
        int x = e.getX();
        int y = e.getY();
	 	
        try {
            int row = gameDisplay.getRow(x);
            int column = gameDisplay.getColumn(y);
            
            game.placePiece(game.getCurrentPlayer(),row, column);
        } catch (IllegalMoveException ex) {
            // warn user
        }
    }
    public void mouseEntered(MouseEvent e)
    {
     	
    }
    public void mouseExited(MouseEvent e)
    {
     	
    }
    public void mousePressed(MouseEvent e)
    {
     	
    }
     
    public void mouseReleased(MouseEvent e) 
    {
     	
    }
}
