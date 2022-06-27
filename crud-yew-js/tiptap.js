let tiptapEditors = new Map();

function getEditor(id) {
  return tiptapEditors.get(id)
}

export function create(id, content, editable, onChange, onSelection) {
  var editor = new window.TipTap.Editor({
    element: document.getElementById(id),
    editable: editable,
    extensions: [
      window.TipTapStarterKit,
      window.TipTapTextAlign.TextAlign.configure({
        types: ['heading', 'paragraph'],
      }),
      window.TipTapHighlight.Highlight,
      window.TipTapImage.Image
    ],
    content: content,
    onUpdate: ({ editor }) => {
      const html = editor.getHTML();
      onChange(html);
    },
    onSelectionUpdate: ({ _editor }) => {
      onSelection();
    },
  });
  tiptapEditors.set(id, editor);
}

export function getHTML(id) {
  return getEditor(id).getHTML();
}

export function isEditable(id) {
  return getEditor(id).isEditable
}

export function toggleHeading(id, level) {
  getEditor(id).chain().focus().toggleHeading({ level: level }).run();
}

export function setParagraph(id) {
  getEditor(id).chain().focus().setParagraph().run();
}

export function toggleBold(id) {
  getEditor(id).chain().focus().toggleBold().run();
}

export function toggleItalic(id) {
  getEditor(id).chain().focus().toggleItalic().run();
}

export function toggleStrike(id) {
  getEditor(id).chain().focus().toggleStrike().run();
}

export function toggleBlockquote(id) {
  getEditor(id).chain().focus().toggleBlockquote().run();
}

export function toggleHighlight(id) {
  getEditor(id).chain().focus().toggleHighlight().run();
}

export function setTextAlignLeft(id) {
  getEditor(id).chain().focus().setTextAlign('left').run();
}

export function setTextAlignCenter(id) {
  getEditor(id).chain().focus().setTextAlign('center').run();
}

export function setTextAlignRight(id) {
  getEditor(id).chain().focus().setTextAlign('right').run();
}

export function setTextAlignJustify(id) {
  getEditor(id).chain().focus().setTextAlign('justify').run();
}

export function setImage(id, src, alt, title) {
  getEditor(id).chain().focus().setImage({ src: src, alt: alt, title: title }).run();
}

function getEditorState(editor) {
  return {
    h1: editor.isActive('heading', { level: 1 }),
    h2: editor.isActive('heading', { level: 2 }),
    h3: editor.isActive('heading', { level: 3 }),
    h4: editor.isActive('heading', { level: 4 }),
    h5: editor.isActive('heading', { level: 5 }),
    h6: editor.isActive('heading', { level: 6 }),
    paragraph: editor.isActive('paragraph'),
    bold: editor.isActive('bold'),
    italic: editor.isActive('italic'),
    strike: editor.isActive('strike'),
    blockquote: editor.isActive('blockquote'),
    highlight: editor.isActive('highlight'),
    align_left: editor.isActive({ textAlign: 'left' }),
    align_center: editor.isActive({ textAlign: 'center' }),
    align_right: editor.isActive({ textAlign: 'right' }),
    align_justify: editor.isActive({ textAlign: 'justify' }),
  }
}

export function getState(id) {
  const editor = getEditor(id);
  return getEditorState(editor);
}