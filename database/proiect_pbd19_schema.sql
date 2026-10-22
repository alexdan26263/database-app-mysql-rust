-- Proiect PBD tema nr. 19

CREATE DATABASE IF NOT EXISTS proiect_pbd19
  CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE proiect_pbd19;

CREATE TABLE IF NOT EXISTS Clienti (
    ClientID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    Nume VARCHAR(100) NOT NULL,
    Prenume VARCHAR(100) NOT NULL,
    Adresa VARCHAR(255) NOT NULL
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS ProdusAlimentar (
    ProdusID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    Denumire VARCHAR(150) NOT NULL,
    DataProducere DATE NOT NULL,
    DataExpirare DATE NOT NULL,
    CONSTRAINT chk_perioada_produs CHECK (DataExpirare >= DataProducere)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS Producatori (
    ProducatorID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    Denumire VARCHAR(150) NOT NULL,
    TaraOrigine VARCHAR(100) NOT NULL,
    Adresa VARCHAR(255) NOT NULL
) ENGINE=InnoDB;

-- Asociere M:N intre Clienti si ProdusAlimentar.
CREATE TABLE IF NOT EXISTS ClientiProduse (
    ClientID INT UNSIGNED NOT NULL,
    ProdusID INT UNSIGNED NOT NULL,
    PRIMARY KEY (ClientID, ProdusID),
    CONSTRAINT fk_cp_client FOREIGN KEY (ClientID)
        REFERENCES Clienti(ClientID)
        ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_cp_produs FOREIGN KEY (ProdusID)
        REFERENCES ProdusAlimentar(ProdusID)
        ON UPDATE CASCADE ON DELETE CASCADE
) ENGINE=InnoDB;

-- Asociere M:N intre ProdusAlimentar si Producatori.
CREATE TABLE IF NOT EXISTS ProduseProducatori (
    ProdusID INT UNSIGNED NOT NULL,
    ProducatorID INT UNSIGNED NOT NULL,
    PRIMARY KEY (ProdusID, ProducatorID),
    CONSTRAINT fk_pp_produs FOREIGN KEY (ProdusID)
        REFERENCES ProdusAlimentar(ProdusID)
        ON UPDATE CASCADE ON DELETE CASCADE,
    CONSTRAINT fk_pp_producator FOREIGN KEY (ProducatorID)
        REFERENCES Producatori(ProducatorID)
        ON UPDATE CASCADE ON DELETE CASCADE
) ENGINE=InnoDB;

-- Conturile aplicatiei sunt DISTINCTE de persoanele din tabela Clienti.
-- Parolele se introduc doar ca hash Argon2id generat de backend, niciodata in clar.
CREATE TABLE IF NOT EXISTS Utilizatori (
    UtilizatorID INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    Username VARCHAR(80) NOT NULL UNIQUE,
    PasswordHash VARCHAR(255) NOT NULL,
    Rol ENUM('client', 'administrator') NOT NULL DEFAULT 'client',
    CreatLa TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB;

-- Interogari de vizualizare a tabelelor de legatura cu datele referite.
CREATE OR REPLACE VIEW V_ClientiProduse AS
SELECT
    cp.ClientID,
    cp.ProdusID,
    c.Nume,
    c.Prenume,
    c.Adresa AS AdresaClient,
    p.Denumire AS DenumireProdus,
    p.DataProducere,
    p.DataExpirare
FROM ClientiProduse cp
JOIN Clienti c ON c.ClientID = cp.ClientID
JOIN ProdusAlimentar p ON p.ProdusID = cp.ProdusID;

CREATE OR REPLACE VIEW V_ProduseProducatori AS
SELECT
    pp.ProdusID,
    pp.ProducatorID,
    p.Denumire AS DenumireProdus,
    p.DataProducere,
    p.DataExpirare,
    pr.Denumire AS DenumireProducator,
    pr.TaraOrigine,
    pr.Adresa AS AdresaProducator
FROM ProduseProducatori pp
JOIN ProdusAlimentar p ON p.ProdusID = pp.ProdusID
JOIN Producatori pr ON pr.ProducatorID = pp.ProducatorID;

-- DATE DE DEMONSTRATIE: ruleaza separat doar pe o baza noua / goala.
-- INSERT INTO Clienti (Nume, Prenume, Adresa) VALUES
-- ('Popescu', 'Ana', 'Bucuresti'),
-- ('Ionescu', 'Mihai', 'Brasov');
-- INSERT INTO ProdusAlimentar (Denumire, DataProducere, DataExpirare) VALUES
-- ('Paine integrala', '2026-10-20', '2026-10-27'),
-- ('Lapte pasteurizat', '2026-10-19', '2026-10-29');
-- INSERT INTO Producatori (Denumire, TaraOrigine, Adresa) VALUES
-- ('Brutaria Exemplu', 'Romania', 'Bucuresti'),
-- ('Lactate Exemplu', 'Romania', 'Cluj-Napoca');
-- INSERT INTO ClientiProduse (ClientID, ProdusID) VALUES (1,1),(1,2),(2,2);
-- INSERT INTO ProduseProducatori (ProdusID, ProducatorID) VALUES (1,1),(2,2);
-- SELECT * FROM V_ClientiProduse;
-- SELECT * FROM V_ProduseProducatori;
