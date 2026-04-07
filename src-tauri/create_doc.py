from docx import Document  
doc = Document()  
doc.add_heading('绵竹市调研报告', 0)  
doc.save('绵竹市调研报告.docx')  
print('文件已保存')  
