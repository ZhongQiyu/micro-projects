package cs260.game_model;

import java.util.Iterator;
import java.util.Vector;

/**
 * @author cassa
 */
public class TicTacToeGame {
    private char [] board;
    private char turn;
    private Vector<TicTacToeListener> listeners;
	
    public TicTacToeGame(char firstPlayer)
    {
        board = new char[9];
		
        for (int i = 0; i < 9; i++) {
            board[i] = ' ';
        }
        turn = firstPlayer;
		
        listeners = new Vector<TicTacToeListener>();
    }
	
    public void placePiece(char player, int x, int y)
            throws IllegalMoveException
    {
        if (player != 'X' && player != 'O') {
            throw new IllegalMoveException("Unknown player " 
                                           + player);	
        } 
        if (turn != player) {
            throw new IllegalMoveException("Player " 
                                           + player + " tried to move on player "
                                           + otherPlayer(player) + "'s turn.");
        }
        int index = getIndex(x, y);
        if (board[index] != ' ') {
            throw new IllegalMoveException("Tried to place a "
                                           + "piece in a non-empty square."); 
        }
		
        board[index] = player;
        if (! gameIsWon(player)) {
            turn = otherPlayer(player);
        }
		
        notifyListeners();
    }
	
    public void addListener(TicTacToeListener l)
    {
        listeners.add(l);
    }
	
    public void removeListener(TicTacToeListener l)
    {
        listeners.remove(l);
    }
	
    private void notifyListeners()
    {
        for (TicTacToeListener l : listeners) {
            l.update();
        }
    }
	
    public char getCurrentPlayer() 
    {
        return turn;
    }
	
    public boolean gameIsWon(char player) 
    {
        for (int a = 0; a < 3; a++) {
            if (boardHasSequence(a, 0, 0, 1, player)
                || boardHasSequence(0, a, 1, 0, player)) {
                return true;
            }
        }
        if (boardHasSequence(0, 0, 1, 1, player)
            || boardHasSequence(2, 0, -1, 1, player)) {
            return true;
        }
		
        return false;
    }
	
    public boolean boardIsFilled()
    {
        for (int i = 0; i < 9; i++) {
            if (board[i] == ' ') {
                return false;
            }
        }
		
        return true;
    }
	
    public String toString()
    {
        String result = "";
		
        for (int i = 0; i < 3; i++) {
            if (i > 0) {
                result += "\n-----\n";
            }
            for (int j = 0; j < 3; j++) {
                try {
                    result += getCell(i, j);
                } catch (IllegalMoveException e) {
                    throw new RuntimeException(e);
                }
                if (j < 2) {
                    result += "|";
                }
            }
        }
        result += "\n" + turn + "'s turn";
		
        return result;
    }
	
    private boolean boardHasSequence(int startX, int startY,
                                     int deltaX, int deltaY,
                                     char player)
    {
        try {
            int x = startX;
            int y = startY;
            for (int count = 0; count < 3; count++) {
                if (getCell(x, y) != player) {
                    return false;
                }
                x += deltaX;
                y += deltaY;
            }
            return true;
        } catch (IllegalMoveException e) {
            throw new RuntimeException(e);
        }
    }
	
    private char otherPlayer(char player) 
    {
        if (player == 'X') {
            return 'O';
        } else {
            return 'X';
        }
    }
	
    private int getIndex(int x, int y)
            throws IllegalMoveException
    {
        if (x < 0 || x > 2 || y < 0 || y > 2) {
            throw new IllegalMoveException("Trying to place "
                                           + "outside board.");
        } 
		
        return x * 3 + y;
    }
	
    public char getCell(int x, int y)
            throws IllegalMoveException
    {
        return board[getIndex(x,y)];
    }
}
