USE proiect_pbd19;

CREATE TABLE IF NOT EXISTS Sesiuni (
    TokenHash CHAR(64) CHARACTER SET ascii COLLATE ascii_bin PRIMARY KEY,
    UtilizatorID INT UNSIGNED NOT NULL,
    ExpiraLa DATETIME NOT NULL,
    CreatLa DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_sesiuni_utilizator (UtilizatorID),
    INDEX idx_sesiuni_expirare (ExpiraLa),
    CONSTRAINT fk_sesiuni_utilizatori FOREIGN KEY (UtilizatorID)
        REFERENCES Utilizatori(UtilizatorID) ON DELETE CASCADE
) ENGINE=InnoDB;

