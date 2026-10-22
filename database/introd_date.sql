USE proiect_pbd19;

INSERT INTO Clienti (Nume, Prenume, Adresa)
VALUES
('Popescu', 'Andrei', 'Bucuresti'),
('Ionescu', 'Maria', 'Brasov'),
('Georgescu', 'Alex', 'Cluj-Napoca'),
('Dumitrescu', 'Elena', 'Constanta'),
('Stan', 'Mihai', 'Timisoara');

INSERT INTO ProdusAlimentar
(Denumire, DataProducere, DataExpirare)
VALUES
('Lapte', '2026-10-01', '2026-10-30'),
('Paine', '2026-10-20', '2026-10-25'),
('Iaurt', '2026-10-10', '2026-11-05'),
('Branza', '2026-10-05', '2026-12-01'),
('Ciocolata', '2026-09-15', '2027-06-15');

INSERT INTO Producatori
(Denumire, TaraOrigine, Adresa)
VALUES
('Lactate SRL', 'Romania', 'Cluj-Napoca'),
('Panificatie SA', 'Romania', 'Bucuresti'),
('Alimente Naturale', 'Romania', 'Brasov'),
('Dulciuri International', 'Germania', 'Berlin');

INSERT INTO ClientiProduse (ClientID, ProdusID)
SELECT c.ClientID, p.ProdusID
FROM (
    SELECT 'Popescu' AS Nume, 'Andrei' AS Prenume, 'Lapte' AS Produs
    UNION ALL SELECT 'Popescu', 'Andrei', 'Paine'
    UNION ALL SELECT 'Ionescu', 'Maria', 'Iaurt'
    UNION ALL SELECT 'Georgescu', 'Alex', 'Ciocolata'
    UNION ALL SELECT 'Dumitrescu', 'Elena', 'Branza'
    UNION ALL SELECT 'Stan', 'Mihai', 'Lapte'
) AS date_test
JOIN Clienti c
  ON c.Nume = date_test.Nume
 AND c.Prenume = date_test.Prenume
JOIN ProdusAlimentar p
  ON p.Denumire = date_test.Produs;

INSERT INTO ProduseProducatori (ProdusID, ProducatorID)
SELECT p.ProdusID, pr.ProducatorID
FROM (
    SELECT 'Lapte' AS Produs, 'Lactate SRL' AS Producator
    UNION ALL SELECT 'Paine', 'Panificatie SA'
    UNION ALL SELECT 'Iaurt', 'Lactate SRL'
    UNION ALL SELECT 'Branza', 'Alimente Naturale'
    UNION ALL SELECT 'Ciocolata', 'Dulciuri International'
) AS date_test
JOIN ProdusAlimentar p
  ON p.Denumire = date_test.Produs
JOIN Producatori pr
  ON pr.Denumire = date_test.Producator;