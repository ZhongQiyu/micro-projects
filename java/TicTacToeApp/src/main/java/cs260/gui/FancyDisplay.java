package cs260.gui;

import java.awt.Dimension;
import java.awt.Font;
import java.awt.FontMetrics;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.BasicStroke;

import javax.swing.JComponent;
import javax.swing.JOptionPane;

import cs260.game_model.TicTacToeGame;
import cs260.game_model.TicTacToeListener;
import cs260.game_model.IllegalMoveException;

/**
 * @author cassa
 */
public class FancyDisplay
		extends JComponent
		implements TicTacToeListener
{
    private TicTacToeGame game;
	private TicTacToeControl mouseController;
	
    private static int WIDTH = 200;
    private static int GAP = 20;
    private static int LINE_WIDTH = 5;
    
    public FancyDisplay(TicTacToeGame game)
    {
        this.game = game;
		
		mouseController = new TicTacToeControl(game, this);
		
        setSize(new Dimension(WIDTH*3, WIDTH*3));
        setPreferredSize(new Dimension(WIDTH*3, WIDTH*3));
    }

    public void paintComponent(Graphics g)
    {
        super.paintComponent(g);

        setLineWidth(g);

        drawLines(g);
        
        try {
            char piece;
		
            for (int i= 0; i < 3; i++) {
                for (int j = 0; j < 3; j++) {
                    piece = game.getCell(i, j);
                    if (piece == 'X') {
                        drawX(g, i, j);
                    } else if (piece == 'O') {
                        drawO(g, i, j);
                    }
                }
            }
        } catch (IllegalMoveException e) {
            throw new RuntimeException(e);
        }
    }

    public void showWinner(char winner)
    {
        JOptionPane.showMessageDialog(this, 
                                      winner + " WINS!!!");
    }

    public void showTieGame()
    {
        JOptionPane.showMessageDialog(this, 
                                      "NOBODY WINS!!!");
    }

    public int getRow(int xPixel)
    {
        return xPixel/WIDTH;
    }

    public int getColumn(int yPixel)
    {
        return yPixel/WIDTH;
    }

    private void setLineWidth(Graphics g)
    {
        ((Graphics2D) g).setStroke(new BasicStroke(LINE_WIDTH));
    }
    
    private void drawLines(Graphics g)
    {
        g.drawLine(0, WIDTH, WIDTH*3, WIDTH);	
        g.drawLine(WIDTH, 0, WIDTH, WIDTH*3);	
        g.drawLine(0, WIDTH*2, WIDTH*3, WIDTH*2);	
        g.drawLine(WIDTH*2, 0, WIDTH*2, WIDTH*3);
    }

    private void drawX(Graphics g, int i, int j)
    {
        g.drawLine(i*WIDTH+GAP, j*WIDTH+GAP,
                   i*WIDTH+WIDTH-GAP, j*WIDTH+WIDTH-GAP);
        g.drawLine(i*WIDTH+WIDTH-GAP, j*WIDTH+GAP,
                   i*WIDTH+GAP, j*WIDTH+WIDTH-GAP);
    }

    private void drawO(Graphics g, int i, int j)
    {
        g.drawOval(i*WIDTH + GAP, j*WIDTH + GAP, WIDTH-2*GAP, WIDTH-2*GAP);
    }

	public void update()
	{
		repaint();
	}
}
