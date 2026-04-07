const md = require('markdown-it')();
const fs = require('fs');

md.renderer.rules.table_open = () => '<div class="table-scroll"><table>';
md.renderer.rules.table_close = () => '</table></div>';

const text = fs.readFileSync('test-table.md', 'utf8');
console.log('=== 应用规范化规则后的文本 ===');
const normalized = text
  .replace(/([^\n])\n(#{1,6})/g, '$1\n\n$2')
  .replace(/(#{1,6})([^\s#])/g, '$1 $2')
  .replace(/(#{1,6})\s{2,}/g, '$1 ')
  .replace(/([^\n])\n([^\n\s])/g, '$1\n\n$2');
console.log(JSON.stringify(normalized));
console.log('\n=== 渲染后的HTML ===');
console.log(md.render(normalized));
