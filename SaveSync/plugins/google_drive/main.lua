function Info()
	return {
		name = "Google Drive",
		description = "Google Drive binding",
		author = "Zachareee",
		icon_url = "https://upload.wikimedia.org/wikipedia/commons/1/12/Google_Drive_icon_%282020%29.svg",
	}
end

function Init(credentials)
	local gdrive = require("gdrive")
	print("This is real")
	print(gdrive.new({}))
end
