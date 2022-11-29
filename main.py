from tfp import TfpCalculator, parse_profile, parse_transfac, parse_fasta

c = TfpCalculator()
c.default_css_threshold = 0.00
c.default_mss_threshold = 0.00
c.add_from_transfac_file("test_files/transfac.txt")
c.add_from_fasta_file("test_files/fasta.txt")
c.add_from_profile_file("test_files/profile.txt")
res = c.evaluate()

for r in res:
    sequence = r.sequence
    matrix = r.matrix
    pos = r.pos
    strand = r.strand
    css = r.css
    mss = r.mss
    len = r.len
    print(sequence, matrix, pos, strand, css, mss, len)

with open("test_files/fasta.txt") as file:
    fasta = parse_fasta(file.read())
    for f in fasta:
        print("Fasta Sequence: ", f.name)
        # print(f.seq)
with open("test_files/transfac.txt") as file:
    transfac = parse_transfac(file.read())
    for t in transfac:
        print("Transfac: ", t.name, t.matrix)
with open("test_files/profile.txt") as file:
    profile = parse_profile(file.read())
    for p in profile:
        print("Profile: ", p.id, "css:", p.css, "mss:", p.mss)

