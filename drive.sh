test_audio () {
	cd Audio2Markdown
	cargo run -- ../testbench/input/audio.wav -o ../testbench/output/Audio2Markdown.md
	vim ../testbench/output/Audio2Markdown.md

	cd -
}

test_image() {
    cd Image2Base64
    cargo run -- --input ../testbench/input/test.png --output ../testbench/output/base64.md
	vim ../testbench/output/base64.txt
    cd -
}

test_pdf() {
	cd Pdf2Docx
	cargo run -- --input ../testbench/input/test.pdf --output ../testbench/output/test.docx
	cd -
}

test_docx() {
	cd Docx2Markdown
	cargo run ../testbench/output/test.docx ../testbench/output/docx.md
	vim ../testbench/output/docx.md
	cd 
}

