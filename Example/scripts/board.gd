extends Control

const Cell = preload("res://Example/scenes/cell.tscn")

var cells : Array = []
var turn : int = 0
var move_num:int = 0

var is_game_end : bool = false

signal send_move(move:int, num:int)
signal match_finished(result)

func _ready():
	cells = []
	move_num = 0
	turn = 0
	for c in get_children():
		remove_child(c)
	for cell_count in range(9):
		var cell = Cell.instantiate()
		cell.main = self
		add_child(cell)
		cells.append(cell)
		cell.cell_updated.connect(_on_cell_updated)
	set_cells_enable(false)

func _on_cell_updated(cell):
	if turn == 0:
		turn = 0
	else:
		turn = 1
	print(turn)
	var match_result = check_match()
	
	if match_result:
		match_finished.emit(match_result)
	print(match_result)
	
	if match_result:
		is_game_end = true
		start_win_animation(match_result)
		# Send win condition move next
	
	for i in range(len(cells)):
		if cells[i] == cell:
			move_num += 1
			send_move.emit(i, move_num)
			set_cells_enable(false)


func process_move(move:int, num:int) -> bool:
	if num == move_num+1:
		var move_cell = cells[move]
		if move_cell.cell_value == "":
			move_cell.draw_cell()
			set_cells_enable(true)
		return true
	return false

func set_cells_enable(is_enabled: bool):
	for cell in cells:
		if cell.cell_value == "":
			cell.disabled = not is_enabled
		else:
			cell.disabled = true
	print("Set enabled: ", is_enabled)

func _on_restart_button_pressed():
	get_tree().reload_current_scene()

func check_match():
	for h in range(3):
		if cells[0+3*h].cell_value == "X" and cells[1+3*h].cell_value == "X" and cells[2+3*h].cell_value == "X":
			return ["X", 1+3*h, 2+3*h, 3+3*h]
	for v in range(3):
		if cells[0+v].cell_value == "X" and cells[3+v].cell_value == "X" and cells[6+v].cell_value == "X":
			return ["X", 1+v, 4+v, 7+v]
	if cells[0].cell_value == "X" and cells[4].cell_value == "X" and cells[8].cell_value == "X":
		return ["X", 1, 5, 9]
	elif cells[2].cell_value == "X" and cells[4].cell_value == "X" and cells[6].cell_value == "X":
		return ["X", 3, 5, 7]
	
	for h in range(3):
		if cells[0+3*h].cell_value == "O" and cells[1+3*h].cell_value == "O" and cells[2+3*h].cell_value == "O":
			return ["O", 1+3*h, 2+3*h, 3+3*h]
	for v in range(3):
		if cells[0+v].cell_value == "O" and cells[3+v].cell_value == "O" and cells[6+v].cell_value == "O":
			return ["O", 1+v, 4+v, 7+v]
	if cells[0].cell_value == "O" and cells[4].cell_value == "O" and cells[8].cell_value == "O":
		return ["O", 1, 5, 9]
	elif cells[2].cell_value == "O" and cells[4].cell_value == "O" and cells[6].cell_value == "O":
		return ["O", 3, 5, 7]
	
	var full = true
	for cell in cells:
		if cell.cell_value == "":
			full = false
	
	if full: return["Draw", 0, 0, 0]

func start_win_animation(match_result: Array):
	var color: Color
	
	if match_result[0] == "X":
		color = Color.BLUE
	elif match_result[0] == "O":
		color = Color.RED
	
	for c in range(3):
		cells[match_result[c+1]-1].glow(color)
