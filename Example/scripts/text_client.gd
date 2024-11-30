extends VBoxContainer

@onready var chat_box = $chat_box
@onready var chat_message = $chat_message
signal new_text_to_send(text: String)
	

func _on_chat_message_text_submitted(new_text: String) -> void:
	if new_text:
		new_text_to_send.emit(new_text)
	chat_message.clear()
	

func add_chat_message(short_name: String, msg):
	chat_box.text += str(short_name, ": ", msg, "\n")
	chat_box.scroll_vertical = chat_box.get_line_height()
