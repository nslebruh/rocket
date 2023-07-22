CREATE PROCEDURE ModifyThreadAmount(IN p_UserId INT, IN p_Floss INT, IN p_Name VARCHAR(255), IN p_Color VARCHAR(512), IN p_Increment BOOLEAN)
BEGIN
    DECLARE v_Amount INT;

    SELECT Amount INTO v_Amount 
    FROM threads 
    WHERE UserId = p_UserId AND Floss = p_Floss;

    IF v_Amount IS NULL AND p_Increment THEN 
        INSERT INTO threads(UserId, Floss, Amount, Name, Color) 
        VALUES (p_UserId, p_Floss, 1, p_Name, p_Color); 
    ELSEIF v_Amount IS NOT NULL AND p_Increment THEN 
        UPDATE threads 
        SET Amount = Amount + 1 
        WHERE UserId = p_UserId AND Floss = p_Floss;
    ELSEIF v_Amount > 1 THEN 
        UPDATE threads 
        SET Amount = Amount - 1 
        WHERE UserId = p_UserId AND Floss = p_Floss;
    ELSEIF v_Amount = 1 THEN
        DELETE FROM threads 
        WHERE UserId = p_UserId AND Floss = p_Floss;
    END IF;

END;
