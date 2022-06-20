let tiptapEditors = new Map();

function getEditor(id) {
  return tiptapEditors.get(id)
}

export function create(id, content, editable, onchange) {
  var editor = new window.TipTap.Editor({
    element: document.querySelector('#' + id),
    editable: editable,
    extensions: [
      window.TipTapStarterKit,
      window.TipTapImage.Image
    ],
    content: content,
    onUpdate: ({ editor }) => {
      const html = editor.getHTML();
      onchange(html);
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

export function setImage(id, src, alt, title) {
  getEditor(id).chain().focus().setImage({ src: src, alt: alt, title: title }).run();
}
